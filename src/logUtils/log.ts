import { invoke } from "@tauri-apps/api/tauri"

export enum LogLevels {
    Info = 0,
    Warning = 1,
    Error = 2
}

type Log = {
    user_id: String,
    log_level: String,
    message: String,
    source: String | null,
}

// private function to convert log level enum because of serialization in rust
function _convertLogLevel(loglevel: number) {
    switch (loglevel) {
        case 0:
            return "Info"
        case 1:
            return "Warning"
        case 2:
            return "Error"
        default:
            return "Unknown"
    }
}

function log(userId: String, logLevel: LogLevels, message: String, source: String | null = null) {

    let magnusLog: Log = {
        user_id: userId,
        log_level: _convertLogLevel(logLevel),
        message: message,
        source: source,
    }

    invoke("create_log", { log: magnusLog })
}

export default log;