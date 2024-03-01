import { exec } from 'child_process';
const platform = process.env.TAURI_PLATFORM;

console.log("Tauri Platform = " + platform)

let commands;
if (platform === 'macos' || platform === 'darwin') {
    commands = [
    "cp src-tauri/libvosk.dylib src-tauri/target/release",
    "cp src-tauri/vosk_api.h src-tauri/target/release"
    ]

    for (let command in commands) {
        exec(commands[command], (error, stdout, stderr) => {
            if (error) {
                console.error(`Error: ${error.message}`);
                return;
            }
            if (stderr) {
                console.error(`Stderr: ${stderr}`);
                return;
            }
            console.log(`Stdout: ${stdout}`);
        });
    }

}
else if (platform === 'windows') {
    commands = [
    "copy .\\src-tauri\\libvosk.lib .\\src-tauri\\target\\release",
    "copy .\\src-tauri\\*.dll .\\src-tauri\\target\\release",
    "copy .\\src-tauri\\vosk_api.h .\\src-tauri\\target\\release"
    ]

    for (let command in commands) {
        exec(commands[command], (error, stdout, stderr) => {
            if (error) {
                console.error(`Error: ${error.message}`);
                return;
            }
            if (stderr) {
                console.error(`Stderr: ${stderr}`);
                return;
            }
            console.log(`Stdout: ${stdout}`);
        });
    }
}