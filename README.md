# watcher

the offline irc bot

...The description up there is kind of a joke; the idea is that you can be offline and still be using the bot because it's able to send you notifications via your phone or your email or whatever.

## Roadmap

Next on the list is probably admin authentication. Right now, the bot decides whether or not it will listen to you based on whether or not it thinks it already knows you--based on the admins it finds in its config file. This isn't the most secure solution, for a host of what should be fairly obvious reasons (...or basically just because IRC does a horrible job of actually authenticating people). I'd like an authentication mechanism based on providing some kind of password to the bot when you give it an admin-only command, but I don't know exactly how I'm going to do that just yet... It's possible that there could be several mechanisms of varying levels of pickiness that we could choose between in configuration.
