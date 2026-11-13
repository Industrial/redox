// Voltaic for VSCode and Cursor. Plan 49 leaf 3.
//
// Plain JavaScript on purpose. A TypeScript source would need a compile step,
// a `tsconfig.json` and a `node_modules`, and would put a build between a
// checkout and a working editor — for a file whose whole job is to start a
// process and hand VSCode its stdio. Plan 49 §1.3 declined an npm dependency
// in the gate for the same reason.
//
// Everything the extension does beyond this file is the language server's:
// diagnostics, document symbols, semantic tokens and formatting all come from
// plan 40's `voltaic-lsp`, which is editor-agnostic by construction. The
// TextMate grammar beside this file exists only so a cold open is coloured
// before the server has started.

const vscode = require("vscode");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

/** @type {LanguageClient | undefined} */
let client;

function serverOptions() {
	const config = vscode.workspace.getConfiguration("voltaic");
	const command = config.get("server.path") || "voltaic-lsp";
	const run = { command, transport: TransportKind.stdio };
	// No separate debug server: there is one binary and no debug build of the
	// protocol, so claiming two would be a lie in a field VSCode logs.
	return { run, debug: run };
}

function activate(context) {
	if (!vscode.workspace.getConfiguration("voltaic").get("server.enable")) {
		return;
	}

	client = new LanguageClient(
		"voltaic-lsp",
		"Voltaic Language Server",
		serverOptions(),
		{
			documentSelector: [{ scheme: "file", language: "voltaic" }],
			synchronize: {
				// `voltaic check` reads Cargo.toml to find the project, so a change to
				// it can change what the server reports.
				fileEvents: vscode.workspace.createFileSystemWatcher("**/Cargo.toml"),
			},
		},
	);

	// `start()` rejects when the binary is missing, which is the common first-run
	// failure. Reported as a message naming the setting, rather than left in the
	// output channel where nobody looks.
	client.start().catch((err) => {
		vscode.window.showErrorMessage(
			`Voltaic: could not start the language server (${err.message}). ` +
				"Install it with `cargo install --path crates/voltaic-lsp`, or set " +
				"`voltaic.server.path`.",
		);
	});

	context.subscriptions.push({
		dispose() {
			if (client) {
				client.stop();
			}
		},
	});
}

function deactivate() {
	return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
