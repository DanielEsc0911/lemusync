import 'package:flutter/material.dart';

import '../design/licenses.dart';
import 'gallery_page.dart';

/// Entry point of the design gallery:
/// `flutter run -d linux -t lib/gallery/main.dart`.
void main() {
  registerBrandLicenses();
  runApp(const GalleryApp());
}
