#!/usr/bin/env bash
# M2 soak sampler for the installed lemusyncd (Linux). Reads /proc, systemctl and the
# daemon's own log; never talks to the daemon (an IPC call would wake it and log a line).
#
#   soak.sh start    sample every 10 minutes (transient systemd user timer; gone at reboot,
#                    run start again)
#   soak.sh stop     stop sampling (the CSV stays)
#   soak.sh sample   take one sample now (what the timer runs)
#   soak.sh report   summarise the CSV: uptime, CPU %, RSS, restarts, syncs, warnings
set -euo pipefail

out="${XDG_STATE_HOME:-$HOME/.local/state}/lemusync/soak.csv"
log="${XDG_DATA_HOME:-$HOME/.local/share}/lemusync/logs/lemusyncd.log"
unit=lemusync-soak

sample() {
  mkdir -p "$(dirname "$out")"
  [[ -s $out ]] || echo "time,pid,ticks,clk_tck,rss_kb,hwm_kb,restarts,state,syncs_ok,syncs_failed,warn_error" >"$out"
  local pid ticks=0 rss=0 hwm=0 restarts state ok=0 failed=0 bad=0
  pid=$(pgrep -xo lemusyncd || true)
  if [[ -n $pid && -r /proc/$pid/stat ]]; then
    # Fields 14 and 15 (utime, stime), counted after the ")" that ends the command name.
    ticks=$(sed 's/.*) //' "/proc/$pid/stat" | awk '{print $12 + $13}')
    rss=$(awk '/^VmRSS:/ {print $2}' "/proc/$pid/status")
    hwm=$(awk '/^VmHWM:/ {print $2}' "/proc/$pid/status")
  fi
  restarts=$(systemctl --user show -p NRestarts --value lemusyncd.service 2>/dev/null || echo "")
  state=$(systemctl --user show -p ActiveState --value lemusyncd.service 2>/dev/null || echo "")
  if [[ -r $log ]]; then
    ok=$(grep -c 'sync finished: .* ok' "$log" || true)
    failed=$(grep -c 'sync finished: .* failed' "$log" || true)
    bad=$(grep -cE ' (WARN|ERROR) ' "$log" || true)
  fi
  echo "$(date -u +%FT%TZ),${pid:-},$ticks,$(getconf CLK_TCK),$rss,$hwm,$restarts,$state,$ok,$failed,$bad" >>"$out"
}

report() {
  [[ -s $out ]] || { echo "no samples yet: $out" >&2; exit 1; }
  # CPU % per daemon process (pid), from its first to its last sample; the log counts
  # are cumulative and reset when the log rotates, so only the last row is shown.
  awk -F, 'NR == 1 { next }
    { if (!($2 in t0)) { t0[$2] = $1; k0[$2] = $3; order[++n] = $2 }
      t1[$2] = $1; k1[$2] = $3; hz = $4; if ($6 > peak) peak = $6; last = $0; rows++ }
    function secs(ts,  c) { gsub(/[-:TZ]/, " ", ts); return mktime(ts, 1) }
    END {
      printf "samples: %d (%s)\n", rows, FILENAME
      for (i = 1; i <= n; i++) { p = order[i]; if (p == "") continue
        d = secs(t1[p]) - secs(t0[p])
        printf "pid %s: %s to %s, %.1f h, CPU %.2f %%\n", p, t0[p], t1[p], d / 3600,
          (d > 0 ? 100 * (k1[p] - k0[p]) / hz / d : 0) }
      split(last, f, ",")
      printf "last: RSS %.1f MiB, peak %.1f MiB, restarts %s, state %s\n", f[5] / 1024, peak / 1024, f[7], f[8]
      printf "log now: %s syncs ok, %s failed, %s WARN/ERROR lines\n", f[9], f[10], f[11]
    }' "$out"
}

case "${1:-}" in
  sample) sample ;;
  report) report ;;
  start)
    sample
    systemd-run --user --quiet --unit="$unit" --on-calendar='*:0/10' --timer-property=AccuracySec=1min \
      "$(readlink -f "$0")" sample
    echo "sampling every 10 min into $out (stop: $0 stop)"
    ;;
  stop) systemctl --user stop "$unit.timer" ;;
  *) echo "usage: $0 start|stop|sample|report" >&2; exit 2 ;;
esac
