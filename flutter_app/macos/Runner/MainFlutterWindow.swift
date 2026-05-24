import Cocoa
import FlutterMacOS

class MainFlutterWindow: NSWindow {
  private static var authorizedUsbUrls: [URL] = []

  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    self.contentViewController = flutterViewController
    self.minSize = NSSize(width: 1120, height: 720)
    self.setContentSize(NSSize(width: 1280, height: 820))
    self.center()

    RegisterGeneratedPlugins(registry: flutterViewController)
    registerNativeChannel(flutterViewController)

    super.awakeFromNib()
  }

  private func registerNativeChannel(_ flutterViewController: FlutterViewController) {
    let channel = FlutterMethodChannel(
      name: "keylesspass/native",
      binaryMessenger: flutterViewController.engine.binaryMessenger
    )
    channel.setMethodCallHandler { call, result in
      switch call.method {
      case "chooseUsbDirectory":
        self.chooseUsbDirectory(result: result)
      default:
        result(FlutterMethodNotImplemented)
      }
    }
  }

  private func chooseUsbDirectory(result: @escaping FlutterResult) {
    DispatchQueue.main.async {
      let panel = NSOpenPanel()
      panel.canChooseFiles = false
      panel.canChooseDirectories = true
      panel.allowsMultipleSelection = false
      panel.canCreateDirectories = false
      panel.directoryURL = URL(fileURLWithPath: "/Volumes", isDirectory: true)

      guard panel.runModal() == .OK, let url = panel.url else {
        result(nil)
        return
      }

      if url.startAccessingSecurityScopedResource() {
        MainFlutterWindow.authorizedUsbUrls.append(url)
      }
      result(url.path)
    }
  }
}
