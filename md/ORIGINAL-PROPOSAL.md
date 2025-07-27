We changed to writing this in Rust, obviously, but here is the original proposal so we can make sure we cover it:

I need you to help me plan this project out called RIP. It is going to be an npx package eventually, but is just a bash script - I'd like to break it into segments if possible because it might get long (can we do that with bash?) - Anyway, the main point of this .sh is this:

1.) It is a security and audit / scanning tool to make sure there is no sensitive data leaking inside of a repository - it also does some other basic security checks.

2.) It can be run in a  kind of "auto" mode with default settings (meant to be run by LLM), and it can be passed CLI arguments in the command itself that runs it, and it can also read from a configuration file from wherever it is run, and it can offer to save (after the interactive mode) the configuration in that directory (so that it will load that same configuration next time). The saving doesn't happen in the auto mode, or when running pre-saved configuration, or when passing arguments in the command. There should also be like a "skip config" command and a command to re-summon the interactive mode and rewrite the configuration.

3.) The script will ask for some various things in interactive mode, but have fallback values to the current working directory and some good defaults.

We will need:

the directory to scan (root of repository)

the name of the .env file

Next, we will show an interactive list of all the keys in the .env file (unless they had blank values or were set to "0" or "localhost" for example, don't show those). They can toggle on/off which keys they want to scan the repository for. 

those are required, and then, we will show a list of file-types to check with a lot of defaults for every language. The user should be able to turn on/off different extensions here in an interactive dropdown (we want it to look nice)

Then, differnet directories can be ignored. Same setup an we want to include a good set of default directories from various languages that would normally be added in .gitignore - and an option to load from the .gitignore itself for files to ignore (the .env would not be respected here obviously for the main scan).

Now, we'll scan the repository and produce a /tmp/ log file and show the user the name of files + lines where we found any values from the .env they were looking for.

Then we will ask if the user wants to do a "Web Scan". If they say yes, we will ask for a URL to the repository.

Once provided the URL, we will check if we can publically access the .env - we can use a whole extra script right here to scan for other common vulernabilities like this, and use cURL to hit their URL and determine if they have any other vulnerabilities.

WE should be able to add on successive other scans and features easily in a kind of modular way here in the programming.

We also want to use shellcheck to link the project.

There are also two ascii art files in the directory with the project. We want the bash output to be super colorful and interfactive and we have some WAREZ style ASCII art. 

We have: rip-logo.txt  rip-skull-logo.txt


In the root directory of the repository for this, we want a fake .env file with a fake API_KEY='someval' and then another test.js file with that api key in it, so we ccan easily test it as well using the auto mode.

This also needs a "help" command and should be programmed at a university or higher level. We need it to be presentable and shareable and fully linted and structured properly in the project so we can also make it available via npx for others to benefit from.

If you can think of extra local system checks we can do and web checks we can do, add them in. 

Make sure it is fast and efficient and can actually be beneficial to users.

I'm not sure if bash has libraries or anything, but if there is other stuff we can pull in to make it look even nicer, or a better way to make this in a terminal - I'd also make this in rust if we could and deploy it that way with cargo - unless it is much more difficult than bash. Just let me know what you think, get a project proposal ready with recommendations and such for us and use the web to recommend other technologieis.
