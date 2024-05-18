import { invoke } from "@tauri-apps/api";
import { log, LogLevels } from "./log";
import * as c from 'crypto-js';

export function generateIdHash(input: string | undefined): string {

    // if we have an value, hash it
    if (input) {
        const hash = c.SHA256(input);
        const hashedString = hash.toString(c.enc.Hex);
        return hashedString;
    }
    // if it is undefined return an empty string and the backend will handle the input validation
    return ""
}

export function create_user(username: string | undefined, email: string | undefined) {
    let created = Date.now();

    if (username && email) {
        // create user on our backend if it doesnt exist
        invoke("create_user", {
            user: {
                user_id: generateIdHash(email),
                username: username,
                email: email,
                created_at: created.toLocaleString()
            },
        }).then(() => {
            log(generateIdHash(email), LogLevels.Info, "User successfully logged in");
        });
    }
}