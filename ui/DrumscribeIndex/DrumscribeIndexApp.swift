import SwiftUI

@main
struct DrumscribeIndexApp: App {
    @StateObject private var favoritesStore = FavoritesStore()
    @StateObject private var updateChecker = UpdateChecker()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(favoritesStore)
                .environmentObject(updateChecker)
        }
        .windowStyle(.titleBar)
        .windowToolbarStyle(.unified)
        .defaultSize(width: 900, height: 600)
        .commands {
            CommandGroup(replacing: .newItem) {}
            CommandGroup(replacing: .undoRedo) {}
            CommandGroup(replacing: .pasteboard) {
                Button("Select All") {
                    NSApp.sendAction(#selector(NSResponder.selectAll(_:)), to: nil, from: nil)
                }
                .keyboardShortcut("a")
            }
            CommandGroup(replacing: .textFormatting) {
                Button("Find...") {
                    guard let window = NSApp.keyWindow else { return }
                    func findSearchField(in view: NSView) -> NSSearchField? {
                        if let field = view as? NSSearchField { return field }
                        return view.subviews.lazy.compactMap { findSearchField(in: $0) }.first
                    }
                    let root = window.contentView?.superview ?? window.contentView
                    if let field = root.flatMap({ findSearchField(in: $0) }) {
                        window.makeFirstResponder(field)
                    }
                }
                .keyboardShortcut("f")
            }
            CommandGroup(replacing: .windowList) {}
            CommandGroup(replacing: .windowArrangement) {}
            CommandGroup(after: .appInfo) {
                Button("Check for Updates…") {
                    Task { await updateChecker.check() }
                }
                .disabled(updateChecker.isChecking)
            }
        }
    }

    class AppDelegate: NSObject, NSApplicationDelegate {
        func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
            true
        }

        func applicationDidFinishLaunching(_ notification: Notification) {
            NSWindow.allowsAutomaticWindowTabbing = false
        }
    }

    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
}
