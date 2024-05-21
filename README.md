# Magnus

Leveraging the industry leading AI model from Open AI, [GPT-4](https://openai.com/gpt-4), Magnus is a conversational assistant equipped with abilities such as weather forecasting, coding assistance, and more. With speech recognition and text-to-speech, Magnus enables a natural feeling verbal interaction. Magnus is available on both MacOS and Windows.

---

# Features

* Speech recognition
* Text-to-Speech
* Proficient in various programming languages and frameworks
* Forecast of your current location *(with permission)*
* Forecast of a named location *(within USA)*
* Access to clipboard text *(with permission)*

---

# Preview

The Magnus app offers options to either create an account with us, to sign-in with your Google account, or to preview the app with no sign-in. Currently, the advantage to signing in is that Magnus will have context of the most recent messages in the conversation when responding to each new message, but more advantages are soon to come. The preview is a great option for an initial exploration the app. All options are free and up to you entirely.

---

# Safety

Currently, when attempting to download and install Magnus, your browser and/or system may flag it as unsafe. We are aware of this issue, and are moving towards solutions to this in future releases. We encourage you to look at the source code in this repository if you have any concerns.

---

# Installation

Releases can be found [here](https://github.com/Magnus-Assistant/magnus/releases).

* **Windows x64**
    * The .msi installer is recommended for Windows
* **Apple Silicon**
    * The .dmg is recommended for Apple Silicon

Because of the **Safety** section above it is possible that **MacOS** will "quarantine" the "Magnus.app" file when installed into the Applications folder. When attempting to open Magnus after installation you may be prompted with something like:

```'magnus' is damaged and can’t be opened. You should move it to the Trash.```

The application is not actually damaged, but rather quarantined by the system because its a zip file downloaded from the internet. To resolve this issue and continue using Magnus as normal, a simple unix command can be used to remove the quarantine attribute. This command will only work if Magnus is located in the Applications folder. Changes to the path may be needed if it's stored somewhere else.

```xattr -d com.apple.quarantine /Applications/Magnus.app/```

To run the command simply open the "Terminal" application, paste in the command, and press enter. After is successfully runs, you should be able to open and use Magnus.

