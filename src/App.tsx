import { createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Device } from "./types";
import "./index.css";

function App() {
	const [devices, setDevices] = createSignal<Device[]>([]);
	const [loading, setLoading] = createSignal(false);
	const [error, setError] = createSignal<string | null>(null);
	const [darkMode, setDarkMode] = createSignal(false);

	const discoverDevices = async () => {
		setLoading(true);
		setError(null);

		try {
			const discoveredDevices = await invoke<Device[]>("discover_devices");
			setDevices(discoveredDevices);
		} catch (err) {
			setError(err instanceof Error ? err.message : "Unknown error");
		} finally {
			setLoading(false);
		}
	};

	onMount(() => {
		const darkModeMediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
		setDarkMode(darkModeMediaQuery.matches);

		darkModeMediaQuery.addEventListener("change", (e) => {
			setDarkMode(e.matches);
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
			<ul class="mt-4">
				{devices().map((device) => (
					<li class="border-b border-gray-200 py-2">
						<strong>IP:</strong> {device.ip}, <strong>Hostname:</strong> {device.hostname}
					</li>
				))}
			</ul>
		</div>
	);
}

export default App;
