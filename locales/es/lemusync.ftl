# Textos de LemuSync para Rust (español neutro latinoamericano). Mismas claves que en/lemusync.ftl.

## Ayuda
help-about = Mantiene tus partidas guardadas sincronizadas entre tus dispositivos, con un almacenamiento que es tuyo.
help-about-json = Muestra JSON para programas (las claves siempre van en inglés).
help-init = Crea un repo cifrado nuevo en el almacenamiento que configuraste.
help-join = Une este dispositivo a un repo existente.
help-join-recovery = Usa la clave de recuperación en lugar de la frase de contraseña.
help-backend = Configura dónde se guarda el repo.
help-backend-add = Elige el almacenamiento.
help-backend-add-gdrive = Usa Google Drive (LemuSync solo ve los archivos que creó).
help-backend-add-gdrive-client_id = Tu id de cliente OAuth (tipo "App de escritorio").
help-backend-add-s3 = Usa un bucket compatible con S3 (Cloudflare R2, AWS S3, Backblaze B2…).
help-backend-add-s3-endpoint = El endpoint, por ejemplo https://<id de cuenta>.r2.cloudflarestorage.com
help-backend-add-s3-bucket = El nombre del bucket.
help-backend-add-s3-access_key_id = El id de la clave de acceso. La clave secreta se pide después y se guarda en el llavero del sistema.
help-backend-add-s3-region = La región (R2 usa auto).
help-backend-add-fs = Usa una carpeta local o montada.
help-backend-add-fs-path = La carpeta.
help-backend-login = Vuelve a iniciar sesión en Google Drive.
help-grant = Elige qué carpetas puede leer y escribir LemuSync.
help-grant-add = Permite una carpeta (la carpeta del emulador).
help-grant-add-path = La carpeta.
help-grant-list = Lista las carpetas permitidas.
help-players = Administra los jugadores.
help-players-add = Agrega un jugador.
help-players-add-name = El nombre del jugador.
help-players-list = Lista los jugadores.
help-scan = Busca emuladores, cuentas y partidas guardadas en este dispositivo.
help-accounts = Conecta jugadores con cuentas del emulador en este dispositivo.
help-accounts-map = Usa una cuenta del emulador para un jugador en este dispositivo.
help-accounts-map-player = El nombre del jugador.
help-accounts-map-emulator = El id del emulador, por ejemplo cemu.
help-accounts-map-account = El id de la cuenta, por ejemplo 80000002.
help-accounts-map-keep = Conserva las partidas guardadas de la cuenta (se suben y luego resuelves un conflicto).
help-accounts-map-replace = Reemplaza las partidas guardadas de la cuenta por las más recientes del jugador (las actuales van primero al historial local).
help-accounts-recover = Guarda una partida huérfana como la primera versión de un jugador.
help-accounts-recover-emulator = El id del emulador, por ejemplo cemu.
help-accounts-recover-account = El id de cuenta de la carpeta huérfana.
help-accounts-recover-player = El jugador que recibe esta partida guardada.
help-accounts-create = Crea una cuenta nueva del emulador para un jugador en este dispositivo y restaura ahí sus partidas guardadas.
help-accounts-create-player = El nombre del jugador.
help-accounts-create-emulator = El id del emulador, por ejemplo cemu.
help-accounts-create-name = El nombre visible de la cuenta (por defecto, el del jugador, recortado si no cabe).
help-accounts-create-yes = No pide confirmación.
help-sync = Sincroniza ahora.
help-sync-dry_run = Muestra qué pasaría sin cambiar nada.
help-sync-yes = Confirma la primera sincronización en este dispositivo.
help-status = Muestra la configuración y las partidas guardadas de este dispositivo.
help-history = Muestra el historial de versiones de un jugador para un juego.
help-history-player = El jugador (por defecto, el único jugador del repo).
help-history-emu = Solo busca entre los juegos de este emulador, por ejemplo cemu.
help-history-game = El juego: un alias, su clave o su nombre, por ejemplo botw o 101c9400.
help-restore = Restaura una versión o una copia local (antes se respaldan los archivos actuales).
help-restore-player = El jugador (por defecto, el único jugador del repo).
help-restore-emu = Solo busca entre los juegos de este emulador, por ejemplo cemu.
help-restore-game = El juego: un alias, su clave o su nombre, por ejemplo botw o 101c9400.
help-restore-what = Un id de versión o el nombre de una copia de `lemusync history`.
help-restore-yes = No pide confirmación.
help-doctor = Revisa el llavero del sistema, el almacenamiento y las carpetas del emulador.

## Errores ($detail nunca se traduce: rutas, ids, texto del proveedor)
error-usage = { $detail }
error-config = El archivo de configuración no es válido: { $detail }
error-backend = El almacenamiento devolvió un error. Vuelve a intentarlo; si sigue pasando, ejecuta `lemusync doctor`.
error-backend_auth = Falló el inicio de sesión del almacenamiento. Ejecuta `lemusync backend login`.
error-backend_offline = No se puede conectar con tu almacenamiento. Revisa tu conexión a internet y vuelve a intentarlo. Tus partidas guardadas en este dispositivo están seguras.
error-backend_quota = Tu almacenamiento está lleno. Libera espacio o consigue más y vuelve a intentarlo. Tus partidas guardadas en este dispositivo están seguras.
error-details = Detalles: { $detail }
error-io = Problema con un archivo: { $detail }
error-crypto = Problema de cifrado: { $detail }
error-unlock = La frase de contraseña o la clave de recuperación no es correcta.
error-locked = Ya hay otro proceso de LemuSync en ejecución ({ $detail }).
error-emulator_running = { $detail } está abierto. Ciérralo y vuelve a intentarlo.
error-not_granted = LemuSync no tiene permiso para usar { $detail }. Ejecuta `lemusync grant add <carpeta>`.
error-not_set_up = Todavía falta configurar: { $detail }
error-secrets = Falló el llavero del sistema: { $detail }
error-data = El repo tiene datos no válidos: { $detail }
error-needs_confirmation = Es la primera sincronización en este dispositivo. Primero respalda las partidas guardadas de este dispositivo y luego ejecuta `lemusync sync --yes`.
error-busy = { $detail } cambió mientras se leía. Vuelve a intentarlo cuando el juego esté en reposo.
error-injected = Falla de prueba en { $detail }.
error-needs-choice = Usa --keep o --replace (la cuenta ya tiene partidas guardadas de: { $games }).
error-needs-tty = Esto necesita una terminal para preguntarte: { $what }.

## Errores con clave: error-<código>-<clave>. Tras `clave:` el motor pasa datos ($detail: una ruta, id o nombre).
error-not_set_up-backend = Todavía no hay almacenamiento configurado. Ejecuta `lemusync backend add fs <carpeta>`, `lemusync backend add s3 --endpoint <url> --bucket <nombre> --access-key-id <id>` o `lemusync backend add gdrive --client-id <id>`.
error-not_set_up-s3_key = La clave secreta de S3 no está en el llavero del sistema de este dispositivo. Vuelve a ejecutar `lemusync backend add s3`.
error-not_set_up-gdrive_login = No iniciaste sesión en Google Drive en este dispositivo. Ejecuta `lemusync backend login`.
error-not_set_up-gdrive_backend = Google Drive no es el almacenamiento de este dispositivo. Ejecuta `lemusync backend add gdrive --client-id <id>`.
error-not_set_up-repo = Este dispositivo todavía no tiene un repo. Ejecuta `lemusync init` o `lemusync join`.
error-not_set_up-repo_key = Falta la clave del repo en el llavero del sistema. Vuelve a ejecutar `lemusync join`.
error-usage-repo_exists = El almacenamiento ya tiene un repo. Mejor ejecuta `lemusync join`.
error-usage-repo_race = Otro dispositivo creó un repo en el almacenamiento al mismo tiempo. Mejor ejecuta `lemusync join`.
error-usage-already_joined = Este dispositivo ya usa otro repo.
error-usage-passphrase_too_short = La frase de contraseña necesita al menos 8 caracteres.
error-usage-player_name_invalid = El nombre de un jugador necesita de 1 a 32 caracteres visibles.
error-usage-player_name_taken = Ese nombre de jugador ya está en uso.
error-usage-file_too_large = { $detail } es demasiado grande para sincronizarse.
error-usage-file_name_invalid = { $detail } tiene un nombre de archivo que no se puede sincronizar.
error-usage-grant_path_missing = Esa carpeta no existe.
error-usage-grant_path_not_allowed = No se puede permitir esa carpeta: elige la carpeta del emulador, no / ni tu carpeta personal.
error-usage-invalid_game_or_account = Ese id de juego o de cuenta no es válido.
error-usage-restore_reason_invalid = Nombre de copia no válido: { $detail }
error-usage-unknown_player = No hay ningún jugador { $detail }. Ejecuta `lemusync players add`.
error-usage-no_install = No se encontró { $detail } en este dispositivo. Ejecuta `lemusync scan`.
error-usage-no_account = Este dispositivo no tiene la cuenta del emulador { $detail }. Créala en el emulador y luego ejecuta `lemusync scan`.
error-usage-account_taken = { $detail } ya usa esa cuenta.
error-usage-not_orphaned = { $detail } no es una partida huérfana en este dispositivo.
error-usage-already_has_save = El jugador ya tiene una partida guardada de { $detail }. Mejor recupérala para un jugador nuevo.
error-usage-folder_empty = Esa carpeta de guardado está vacía.
error-usage-not_mapped = El jugador no tiene una cuenta de { $detail } en este dispositivo. Ejecuta `lemusync accounts map`.
error-usage-game_not_found = Ese juego no se encontró en este dispositivo. Ejecuta `lemusync games` para ver los juegos encontrados, o indica su ruta con `lemusync config set { $detail } <ruta>`.
error-usage-unknown_restore_point = { $detail } no es una versión ni una copia. Consulta `lemusync history`.
error-usage-cannot_create = LemuSync no puede crear cuentas de { $detail }. Créala en el emulador y luego ejecuta `lemusync accounts map`.
error-usage-too_many_accounts = El emulador ya tiene su máximo de { $detail } cuentas.
error-usage-account_exists = Ya hay algo llamado { $detail } en la carpeta de cuentas. No se cambió nada.
error-usage-bad_account_name = Ese nombre de cuenta está vacío o tiene caracteres que el emulador no puede guardar.
error-usage-no_free_account_id = No queda ningún id de cuenta libre.
error-config-repo_mismatch = El repo del almacenamiento no es el que usa este dispositivo.
error-config-repo_config = La lista de jugadores del repo no es válida.
error-config-no_home_dir = No se encontró tu carpeta personal.
error-config-invalid_flatpak_app_id = Un perfil de emulador tiene un id de app de Flatpak no válido.
error-config-invalid_regex = Un perfil de emulador tiene un patrón no válido.
error-config-unsupported_capture = Un perfil de emulador usa un patrón no compatible.
error-secrets-bad_master_key = La clave del repo en el llavero del sistema está dañada. Vuelve a ejecutar `lemusync join`.
error-secrets-no_default_collection = El llavero del sistema no tiene una colección predeterminada. LemuSync nunca crea una; configúrala en tu app de llavero.
error-secrets-prompt_needed = El llavero del sistema pidió un diálogo de confirmación. LemuSync nunca abre uno; revisa tu app de llavero.
error-secrets-ambiguous = El llavero del sistema tiene más de una entrada de LemuSync con el mismo nombre. Elimina la que sobra en tu app de llavero.
error-secrets-off_under_test_root = El llavero del sistema está desactivado mientras LEMUSYNC_TEST_ROOT está definido.
error-secrets-bad_reply = El llavero del sistema envió una respuesta que LemuSync no entiende.
error-secrets_unavailable = El llavero del sistema todavía no está disponible ({ $detail }). LemuSync volverá a intentarlo.
error-secrets_unavailable-no_owner = El llavero del sistema todavía no está en ejecución. LemuSync nunca inicia uno; volverá a intentarlo cuando tu sesión de escritorio lo abra.
error-secrets_unavailable-no_bus = Todavía no hay bus de sesión del escritorio, así que no se puede acceder al llavero del sistema. LemuSync volverá a intentarlo.
error-secrets_unavailable-locked = El llavero del sistema está bloqueado. Desbloquéalo (normalmente al iniciar sesión); LemuSync nunca lo desbloquea y volverá a intentarlo.
error-secrets_unavailable-gone = El llavero del sistema se cerró mientras LemuSync lo usaba. LemuSync volverá a intentarlo.
error-data-repo_format_newer = El repo lo creó una versión más nueva de LemuSync. Actualiza LemuSync en este dispositivo.
error-data-history_changed = Una copia del historial local cambió mientras se subía: { $detail }
error-data-blob_corrupt = Un archivo del repo está dañado ({ $detail }).
error-data-manifest_invalid = Una versión del repo no es válida.
error-data-wrong_size = Un archivo del repo tiene un tamaño incorrecto: { $detail }
error-data-file_too_large = Un archivo del repo es demasiado grande: { $detail }

## Preguntas
prompt-passphrase = Frase de contraseña:
prompt-passphrase-again = Escribe la frase de contraseña otra vez:
prompt-passphrases-differ = Las frases de contraseña no coinciden.
prompt-recovery = Clave de recuperación:
prompt-client-secret = Secreto del cliente OAuth (no se muestra):
prompt-s3-secret = Clave de acceso secreta (no se muestra):
prompt-first-sync = Es la primera sincronización en este dispositivo. ¿Respaldaste las partidas guardadas de este dispositivo?
prompt-keep-or-replace = La cuenta { $account } ya tiene partidas guardadas de { $games }. ¿Las conservas (se suben y resuelves un conflicto) o las reemplazas por la partida guardada más reciente de { $player } (los archivos actuales van primero al historial local)?
prompt-keep-or-replace-keys = [c]onservar / [r]eemplazar:
prompt-restore = ¿Reemplazar los archivos actuales de { $player } por { $what }? Los archivos actuales van primero al historial local.
prompt-create-account = ¿Crear la cuenta “{ $name }” de { $emulator } para { $player } en este dispositivo? { $emulator } tiene que estar cerrado.
prompt-yes-no = [s/N]:
answer-yes = s
answer-keep = c
answer-replace = r

## Resultados
login-open-url = Abre esta dirección en tu navegador para que LemuSync use su propia carpeta de Google Drive:
login-waiting = Esperando al navegador (hasta 5 minutos)…
login-done = Google Drive está conectado.
backend-fs-done = El repo se guardará en { $path }.
backend-s3-done = El repo se guardará en el bucket { $bucket } de { $endpoint }.
init-done = Se creó el repo { $repo } en { $device }.
init-recovery = Tu clave de recuperación. Anótala ahora: solo se muestra una vez, y sin ella o sin la frase de contraseña no se pueden recuperar tus partidas guardadas.
join-done = { $device } se unió al repo { $repo }.
grant-done = Permitido: { $path }
grant-none = Todavía no hay carpetas permitidas.
players-added = Se agregó el jugador { $name }.
players-none = Todavía no hay jugadores.
map-done = { $player } ahora usa la cuenta { $account } de { $emulator } en este dispositivo.
recover-done = La partida huérfana ahora es la primera versión de { $game } de { $player }.
create-done = Se creó la cuenta “{ $name }” ({ $account }) de { $emulator } para { $player }. Abre { $emulator } y elige “{ $name }” en la configuración de cuentas.
restore-done = Se restauró { $what } para { $player }. Ejecuta `lemusync sync` para subirla.
sync-line = { $player } · { $game } · { $action }
sync-line-moved = { $player } · { $game } · { $action } · { $files ->
    [one] 1 archivo
   *[other] { $files } archivos
}, { $size } MB
sync-elapsed = Listo en { $secs } s.
progress-checking = { $player } · { $game } · revisando…
progress-uploading = { $player } · { $game } · subiendo { $bar } { $done }/{ $total } · { $size_done }/{ $size_total } MB · { $rate } MB/s
progress-downloading = { $player } · { $game } · descargando { $bar } { $done }/{ $total } · { $size_done }/{ $size_total } MB · { $rate } MB/s
connection-trouble = Problemas de conexión, reintentando…
connection-back = La conexión volvió.
dry-run = Prueba: no se cambió nada.
sync-conflicts = { $count ->
    [one] Una partida guardada tiene un conflicto. Se conservan ambas versiones. Ejecuta `lemusync conflicts`.
   *[other] { $count } partidas guardadas tienen un conflicto. Se conservan todas las versiones. Ejecuta `lemusync conflicts`.
}

action-nothing = al día
action-adopt = ya coincide con el repo
action-fast_forward = se restauró la versión más reciente
action-upload = subida
action-upload_then_conflict = subida, con un conflicto por resolver
action-conflict = conflicto por resolver
action-local_missing = falta la carpeta de la partida guardada
action-remote_behind = el repo volvió a un estado anterior; no se cambió nada
action-needs_account = no hay cuenta en este dispositivo: ejecuta `lemusync accounts map`
action-skipped = omitida (el emulador está abierto)
action-resolved = conflicto resuelto; las otras versiones quedan en el historial

warning-emulator_running = { $detail } está abierto, así que se omitieron sus partidas guardadas.
warning-not_granted = Todavía sin permiso: { $detail }. Ejecuta `lemusync grant add`.
warning-shared_folder_not_empty = { $detail } lo comparten todas las cuentas y todavía no se sincroniza.
warning-orphaned_save = Partida huérfana { $detail }: se borró su cuenta. Ejecuta `lemusync accounts recover` para conservarla.
warning-config_merged = Se combinaron cambios de dos dispositivos.
warning-player_renamed = Dos jugadores tenían el mismo nombre; uno ahora se llama { $detail }.
warning-remote_behind = Al repo le faltan versiones que este dispositivo vio ({ $detail }). No se cambió nada.
warning-restore_rolled_back = Se deshizo una restauración interrumpida en { $detail }.
warning-invalid_version = Se ignoraron datos no válidos del repo: { $detail }
warning-setup_incomplete = Se creó el repo, pero este dispositivo no quedó configurado del todo ({ $detail }). Ejecuta `lemusync join` con tu frase de contraseña para terminar.
warning-unreadable_folder = No se pudo leer una carpeta de guardado de forma segura y se omitió: { $detail }
warning-lease_renew_failed = No se pudo renovar el bloqueo ({ $detail }); se reintenta. El juego sigue abierto.
warning-lease_release_failed = No se pudo liberar el bloqueo ({ $detail }); vence solo.

scan-install = { $emulator } en { $base }
scan-needs-grant = Permítelo: lemusync grant add { $path }
scan-account = cuenta { $id } · { $name } · { $player }
scan-unknown = (ninguno)
scan-save = partida guardada { $game }/{ $account } · { $files ->
    [one] 1 archivo
   *[other] { $files } archivos
}
scan-orphan = partida huérfana { $game }/{ $account }: ejecuta `lemusync accounts recover`

status-device = Dispositivo: { $device }
status-repo = Repo: { $repo } en { $backend }
status-no-repo = Todavía no hay un repo: ejecuta `lemusync init` o `lemusync join`.
status-stream = { $player } · { $emulator } { $game } · cuenta { $account }

history-version = { $id } · { $files ->
    [one] 1 archivo
   *[other] { $files } archivos
} { $head }
history-head = (la más reciente)
history-backup = copia local { $name }

doctor-ok = ok
doctor-fail = FALLÓ
doctor-check = { $status } { $name } { $detail }

help-play = Juega: trae la partida guardada más reciente, toma el bloqueo, abre el emulador y sube la partida al cerrarlo.
help-play-emu = Solo busca entre los juegos de este emulador, por ejemplo cemu.
help-play-game = El juego: un alias, su clave o su nombre, por ejemplo botw o "breath of the wild".
help-play-player = El jugador (por defecto, el único jugador del repo).
help-play-args = Argumentos extra para el emulador, después de --.
help-conflicts = Muestra las partidas guardadas con conflicto y sus versiones.
help-resolve = Resuelve un conflicto conservando una versión (las demás quedan en el historial).
help-resolve-game = El juego: un alias, su clave o su nombre, por ejemplo botw o 101c9400.
help-resolve-version = El id de la versión que se conserva, de `lemusync conflicts`.
help-resolve-player = El jugador (necesario si varios jugadores tienen un conflicto en este juego).
help-lease = Muestra o libera bloqueos (qué dispositivo está jugando qué).
help-lease-status = Muestra todos los bloqueos del repo.
help-lease-release = Borra el bloqueo de este dispositivo en un juego, por ejemplo uno que dejó un cierre inesperado.
help-lease-release-force = Obligatorio: confirma que quieres borrar el bloqueo.
help-lease-release-game = El juego: un alias, su clave o su nombre, por ejemplo botw o 101c9400.

play-pulling = Trayendo la partida guardada más reciente…
play-waiting = Otro dispositivo está jugando. Esperando a que termine…
play-running = { $emulator } está abierto. Tu partida se sube cuando lo cierres.
play-uploading = Se cerró { $emulator }. Subiendo tu partida…
play-ctrl-c = El juego sigue abierto. Ciérralo ahí; luego LemuSync sube tu partida.
play-done = { $emulator } terminó con el código { $code }.
prompt-lease-held = { $device } está jugando esta partida (visto hace { $ago } s).
prompt-lease-stale = { $device } estaba jugando esta partida pero dejó de responder hace { $ago } s.
prompt-lease-keys = [e]sperar / [j]ugar de todos modos (habrá conflicto si ambos guardan):
prompt-lease-keys-stale = [e]sperar / [t]omar el bloqueo / [j]ugar de todos modos:
answer-wait = e
answer-play-anyway = j
answer-take-over = t
prompt-conflict = { $game } tiene un conflicto. ¿Qué versión quieres jugar?
prompt-conflict-pick = Número de versión:
conflicts-none = No hay conflictos.
conflict-stream = { $player } · { $emulator } · { $game }
conflict-head = { $n }. { $version } · { $device } · { $files ->
    [one] 1 archivo
   *[other] { $files } archivos
}
resolve-done = Se conservó { $version } para { $player } · { $game }. Las otras versiones quedan en el historial.
lease-none = No hay bloqueos.
lease-line = { $player } · { $emulator } · { $game } · { $device }, hace { $ago } s{ $ours ->
    [yes] {" "}(este dispositivo)
   *[no] {""}
}
lease-line-stale = { $player } · { $emulator } · { $game } · { $device }, venció hace { $ago } s{ $ours ->
    [yes] {" "}(este dispositivo)
   *[no] {""}
}
lease-released = { $count ->
    [0] Este dispositivo no tiene un bloqueo en { $game }.
    [one] Se liberó el bloqueo de este dispositivo en { $game }.
   *[other] Se liberaron { $count } bloqueos de este dispositivo en { $game }.
}
error-lease_held = { $detail } está jugando esta partida. Ejecuta `lemusync play` en una terminal para esperar o jugar de todos modos.
error-needs_decision = { $detail } tiene un conflicto. Ejecuta `lemusync conflicts` y luego `lemusync resolve`.
error-usage-player_needed = Elige el jugador con --as <nombre>.
error-usage-version_invalid = { $detail } no es un id de versión. Ejecuta `lemusync conflicts` para verlos.
error-usage-no_conflict = { $detail } no tiene conflictos.
error-usage-lease_live = Ese bloqueo está activo; no se puede tomar.

## config (config-and-cli §Commands)
help-config = Lee y cambia los ajustes de este dispositivo (config.toml).
help-config-path = Muestra dónde está el archivo de configuración.
help-config-list = Lista todos los ajustes.
help-config-get = Muestra un ajuste.
help-config-get-key = El ajuste, por ejemplo conflict_policy o emulators.cemu.exe.
help-config-set = Cambia un ajuste. Los comentarios del archivo se conservan.
help-config-set-key = El ajuste, por ejemplo leases.ttl_secs o emulators.cemu.games.101c9500.
help-config-set-values = El valor nuevo. emulators.<id>.exe acepta varias palabras: el comando que abre el emulador.
help-config-unset = Quita un ajuste y vuelve a su valor predeterminado.
help-config-unset-key = El ajuste.
config-unset-value = (sin definir)
config-set-done = { $key } = { $value }
config-unset-done = Se quitó { $key }.
error-usage-config_managed = Ese ajuste se cambia con `{ $detail }`, no con `lemusync config`.
error-usage-config_fixed = { $detail } no se puede cambiar.
error-usage-config_unknown_key = No existe el ajuste { $detail }. Ejecuta `lemusync config list`.
error-usage-config_unknown_emulator = No existe el emulador { $detail }.
error-usage-config_bad_value = Ese no es un valor válido para { $detail }. No se cambió nada.
error-usage-config_one_value = { $detail } acepta un solo valor.

## games
help-games = Muestra los juegos instalados en este dispositivo, con sus nombres y rutas. Es local, no usa el almacenamiento.
games-line = { $emulator } · { $name } · { $key } · { $path } · { $source }
games-none = No se encontraron juegos. Agrega tus carpetas de juegos en el emulador, o agrega una con `lemusync config set emulators.<id>.game_dirs <carpeta>`.
games-no-path = (sin ruta)
games-source-config = de la configuración
games-source-emulator = del emulador
games-source-scan = de game_dirs
help-games-alias = Le da a un juego un nombre corto para escribir en lugar de su nombre o clave.
help-games-alias-game = El juego: su nombre, su clave o un alias.
help-games-alias-alias = El nombre corto: de 1 a 32 caracteres entre a-z, 0-9, - y _.
help-games-alias-emu = Solo busca entre los juegos de este emulador, por ejemplo cemu.
help-games-unalias = Quita el nombre corto de un juego.
help-games-unalias-alias = El nombre corto.
games-line-aliases = { $line } · alias: { $aliases }
games-alias-done = { $alias } ahora es { $game }.
games-unalias-done = Se quitó el alias { $alias }.
error-usage-game_unknown = No hay ningún juego { $detail }. Ejecuta `lemusync games` para ver los juegos de este dispositivo.
error-usage-game_ambiguous = Hay más de un juego que coincide: { $detail }. Agrega --emu <id>, escribe la clave del juego o dale un nombre corto con `lemusync games alias <clave> <alias>`.
error-usage-alias_invalid = { $detail } no puede ser un alias: usa de 1 a 32 caracteres entre a-z, 0-9, - y _, empezando con una letra o un dígito.
error-usage-alias_is_key = { $detail } es la clave de otro juego, así que no puede ser un alias.
error-usage-alias_taken = El alias { $detail } ya lo usa otro juego. Quítalo primero con `lemusync games unalias { $detail }`.
error-usage-alias_unknown = No hay ningún alias { $detail }. Ejecuta `lemusync games` para verlos.
error-usage-play_old_shape = Ahora el emulador va después de --emu: { $detail }
history-game = { $game }
play-starting = Iniciando { $game }…

## Daemon: textos de estado (status-and-notifications §Status model)
status-synced = Todas las partidas están sincronizadas
status-scanning = Buscando cambios
status-uploading = { $count ->
    [one] Subiendo 1 cambio
   *[other] Subiendo { $count } cambios
}
status-downloading = { $count ->
    [one] Descargando 1 cambio
   *[other] Descargando { $count } cambios
}
status-syncing = { $count ->
    [one] Sincronizando 1 cambio
   *[other] Sincronizando { $count } cambios
}
status-paused = Sincronización en pausa
status-offline = { $count ->
    [one] Sin conexión · 1 cambio en espera
   *[other] Sin conexión · { $count } cambios en espera
}
status-conflicts = { $count ->
    [one] 1 conflicto necesita tu decisión
   *[other] { $count } conflictos necesitan tu decisión
}
status-foreign-leases = { $count ->
    [one] 1 juego está en uso en otro dispositivo
   *[other] { $count } juegos están en uso en otros dispositivos
}
status-error-auth = La sesión del almacenamiento expiró
status-error-quota = El almacenamiento está lleno
status-error-permission = LemuSync no puede abrir una carpeta de partidas
status-error-config = El archivo de configuración tiene un problema
status-error-not-set-up = Aún no está configurado: ejecuta lemusync init o lemusync join
status-playing = Jugando { $game } · { $player }

## Daemon: menú de la bandeja
tray-sync-now = Sincronizar ahora
tray-pause = Pausar
tray-pause-1h = Por 1 hora
tray-pause-until-resume = Hasta que la reanude
tray-resume = Reanudar sincronización
tray-recent = Juegos recientes
tray-quit = Salir

## Daemon: notificaciones
notice-conflict-title = Dos dispositivos cambiaron { $game }
notice-conflict-body = Ejecuta lemusync conflicts para elegir qué partida guardada conservar.
notice-foreign-lease-title = { $game } está en uso en { $device }
notice-foreign-lease-body = Elige Esperar y LemuSync te avisa cuando se libere.
notice-auth-body = Vuelve a iniciar sesión en tu almacenamiento para seguir sincronizando.
notice-quota-body = Libera espacio en tu almacenamiento para seguir sincronizando.
notice-action-wait = Esperar
notice-remote-newer-title = Hay una partida más nueva de { $game } en { $device }
notice-remote-newer-body = Cierra { $emulator } y LemuSync la actualiza.

## service (servicio de usuario de systemd en Linux)
help-service = Ejecuta el daemon de sincronización en segundo plano (servicio de usuario de systemd en Linux).
help-service-install = Instala e inicia el daemon; vuelve a iniciarse en cada inicio de sesión.
help-service-install-headless = Sigue sincronizando aunque nadie haya iniciado sesión (activa linger). Así no hay bandeja ni notificaciones.
help-service-uninstall = Detiene y quita el servicio del daemon.
help-service-start = Inicia el daemon.
help-service-stop = Detiene el daemon.
help-service-status = Muestra si el daemon está instalado y en ejecución.
service-installed = El daemon está instalado y en ejecución ({ $path }).
service-headless = Sigue sincronizando después de que cierres sesión.
service-uninstalled = El daemon se detuvo y se quitó.
service-linger-left = Linger quedó como estaba. Para desactivarlo: loginctl disable-linger
service-not-installed = El daemon no está instalado. Ejecuta `lemusync service install`.
service-started = El daemon se inició.
service-stopped = El daemon se detuvo.
service-state = Daemon: { $enabled ->
    [yes] se inicia al iniciar sesión
   *[no] no se inicia al iniciar sesión
}, { $active ->
    [yes] en ejecución
   *[no] detenido
}, se reinició { $restarts } veces.
error-usage-service_unsupported = `lemusync service` por ahora solo funciona en Linux.
error-usage-daemon_not_found = No se encuentra lemusyncd junto a lemusync ni en el PATH.
error-usage-daemon_path_invalid = lemusyncd debe ser un archivo existente con ruta absoluta: { $detail }

## lemusyncd (el daemon)
daemon-already-running = LemuSync ya se está ejecutando para este usuario (otro lemusyncd o comando lemusync usa { $path }).
daemon-start-failed = lemusyncd no pudo iniciar: { $error }

## Daemon: avisos de Jugar y Esperar en la bandeja
notice-lease-free-title = { $game } ya está libre
notice-lease-free-body = El otro dispositivo lo soltó. Juégalo desde el menú de LemuSync.
notice-play-failed-title = No se pudo iniciar { $game }

## CLI con el daemon (pause, resume, logs, status)
help-pause = Pausa la sincronización automática del daemon (hasta reanudarla o por un tiempo).
help-pause-duration = Cuánto tiempo, p. ej. 30m, 1h, 1h30m (como máximo 30 días).
help-resume = Reanuda la sincronización automática del daemon.
help-logs = Muestra el registro del daemon.
help-logs-follow = Sigue mostrando las líneas nuevas.
pause-done = Sincronización en pausa hasta que ejecutes lemusync resume.
pause-done-for = { $minutes ->
    [one] Sincronización en pausa por 1 minuto.
   *[other] Sincronización en pausa por { $minutes } minutos.
}
resume-done = Sincronización reanudada.
status-daemon = Servicio en segundo plano: { $state }
error-daemon = Falló el servicio en segundo plano: { $detail }
error-usage-daemon_not_running = El daemon no está en ejecución, así que no hay nada que pausar ni reanudar. Inícialo con `lemusync service start`.
error-usage-daemon_running_oauth = El inicio de sesión de Google Drive se hace en esta terminal, no en el daemon. Ejecuta `lemusync service stop`, inicia sesión y luego `lemusync service start`.
error-usage-duration_invalid = "{ $detail }" no es una duración. Usa p. ej. 30m, 1h o 1h30m.
error-usage-no_log = El daemon todavía no escribió un registro ({ $detail }).
error-usage-play_arg_not_utf8 = El daemon solo puede pasar argumentos de texto al emulador: { $detail }
error-usage-path_not_utf8 = El daemon solo puede usar rutas de texto: { $detail }
