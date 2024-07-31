import { createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Device } from "./types";
import "./index.css";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import "./utils";
import { checkNotificationsPermission, enqueueNotification } from "./utils";

function App() {
	const [devices, setDevices] = createSignal<Device[]>([]);
	const [loading, setLoading] = createSignal(false);
	const [error, setError] = createSignal<string | null>(null);
	const [darkMode, setDarkMode] = createSignal(false);
	const [selectedDevice, setSelectedDevice] = createSignal<Device | null>(null);
	const [filePaths, setFilePaths] = createSignal<string[]>([]);

	const discoverDevices = async () => {
		setLoading(true);
		setError(null);

		try {
			const discoveredDevices = await invoke<Device[]>("discover_devices");
			console.log(discoveredDevices);
			setDevices(discoveredDevices);
		} catch (err) {
			setError(err instanceof Error ? err.message : "Unknown error");
		} finally {
			setLoading(false);
		}
	};

	const startServer = async () => {
		try {
			await invoke("start_file_server");
		} catch (err) {
			console.error(err);
			setError(err instanceof Error ? err.message : "Unknown error");
		}
	};

	const openFileDialog = async () => {
		const selected = await open({
			multiple: true,
			canCreateDirectories: false,
			directory: false,
			recursive: false,
			title: "Select file(s) to send"
		});
		if (Array.isArray(selected)) {
			setFilePaths(selected.map((file) => file.path));
		} else if (selected !== null) {
			setFilePaths([selected]);
		}
	};

	const uploadFile = async () => {
		const device = selectedDevice();
		const paths = filePaths();
		if (!device || paths.length === 0) return;

		try {
			setLoading(true);
			for (const filePath of paths) {
				await invoke("send_file", { device, filePath });
			}
			enqueueNotification("File Sent", `Sent file(s) to ${device.hostname}`);
			setSelectedDevice(null);
			setFilePaths([]);
		} catch (err) {
			setError(err instanceof Error ? err.message : "Unknown error");
		} finally {
			setLoading(false);
		}
	};

	const closeMenu = () => {
		setSelectedDevice(null);
		setFilePaths([]);
	};

	onMount(() => {
		checkNotificationsPermission();
		startServer();

		const darkModeMediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
		setDarkMode(darkModeMediaQuery.matches);

		darkModeMediaQuery.addEventListener("change", (e) => {
			setDarkMode(e.matches);
		});

		listen("file-received", (event) => {
			enqueueNotification("File Received", `Received file: ${event.payload}`);
			console.log("File received", event.payload);
		});
	});

	return (
		<div class={`p-4 ${darkMode() ? "dark bg-black text-white" : ""} min-h-screen`}>
			<h1 class="mb-4 text-2xl font-bold">Discovered Devices</h1>
			<button
				class={`rounded-md bg-blue-500 px-4 py-2 text-white hover:bg-blue-600 ${darkMode() ? "dark:bg-gray-700 dark:hover:bg-gray-600" : ""}`}
				onClick={discoverDevices}
				disabled={loading()}>
				{loading() ? "Searching..." : "Discover Devices"}
			</button>
			{error() && <p class="mt-2 text-red-500">Error: {error()}</p>}
			<div class="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
				{devices().map((device) => (
					<div
						class="cursor-pointer rounded-lg border border-gray-200 bg-white p-4 shadow-lg hover:bg-gray-100 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-600"
						onClick={() => setSelectedDevice(device)}>
						<h2 class="text-lg font-bold">IP: {device.ip}</h2>
						<p>
							<strong>Hostname:</strong> {device.hostname}
						</p>
						<p>
							<strong>OS:</strong> {device.os}
						</p>
					</div>
				))}
			</div>

			{selectedDevice() && (
				<div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
					<div class="rounded-lg bg-white p-6 shadow-lg dark:bg-gray-900">
						<h2 class="mb-4 text-xl font-bold">Send File to {selectedDevice()!.hostname}</h2>
						<button
							class={`rounded-md bg-blue-500 px-4 py-2 text-white hover:bg-blue-600 ${darkMode() ? "dark:bg-gray-700 dark:hover:bg-gray-600" : ""}`}
							onClick={openFileDialog}>
							{filePaths() ? "Change File" : "Select File"}
						</button>
						<p class="mt-2">{filePaths() ? `Selected file: ${filePaths()}` : "No file selected"}</p>
						<button
							class={`mt-4 rounded-md bg-green-500 px-4 py-2 text-white hover:bg-green-600 ${darkMode() ? "dark:bg-gray-700 dark:hover:bg-gray-600" : ""}`}
							onClick={uploadFile}
							disabled={loading() || !filePaths()}>
							{loading() ? "Uploading..." : "Send File"}
						</button>
						<button
							class={`ml-4 rounded-md bg-red-500 px-4 py-2 text-white hover:bg-red-600 ${darkMode() ? "dark:bg-gray-700 dark:hover:bg-gray-600" : ""}`}
							onClick={closeMenu}>
							Cancel
						</button>
					</div>
				</div>
			)}
		</div>
	);
}

export default App;
