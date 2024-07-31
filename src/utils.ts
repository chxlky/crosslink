import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";

export async function checkNotificationsPermission() {
	if (!(await isPermissionGranted())) {
		return (await requestPermission()) === "granted";
	}
	return true;
}

export async function enqueueNotification(title: string, body: string) {
	if (!(await checkNotificationsPermission())) {
		return;
	}
	sendNotification({ title, body });
}
