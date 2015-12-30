# watcher
the offline irc bot

...The description up there is kind of a joke; the idea is that you can be offline and still be using the bot because it's able to send you notifications via your phone or your email or whatever.

## Roadmap
watcher is pretty closely dependent on a couple different web services, and right now those are all hit synchronously: like, if you go to send a message via Twilio, the bot has to get a response back from the Twilio API before it can do anything else. Realistically, this makes exactly *zero* difference in nominal usage right now, because the bot doesn't actually need to process that many incoming messages, but what I would like to do going forward is offload those responsibilities to another thread.

If you look at the [listener][listener] code, the first thing you should notice is that objects like the `Irc`, the `Channel`, the `ChannelUser`, the incoming message, and... Well, I'm guessing pretty much everything, honestly... Anyway, it all comes in via a shared reference (`&Irc`, for example), which means--I believe--that we can pass those right through to another thread without any problem, which means that this is pretty much trivial: we send that reference on to the thread that's going to do the work, and then that thread can use the `&Irc` reference to send the result of its work back to the channel.

Another thing I'd like to work on is admin authentication. Right now, the bot decides whether or not it will listen to you based on whether or not it thinks it already knows you--based on the admins it finds in its config file. This isn't the most secure solution, for a host of what should be fairly obvious reasons (...or basically just because IRC does a horrible job of actually authenticating people). I'd like an authentication mechanism based on providing some kind of password to the bot when you give it an admin-only command, but I don't know exactly how I'm going to do that just yet... It's possible that there could be several mechanisms of varying levels of pickiness that we could choose between in configuration.

[listener]:https://github.com/archer884/watcher/blob/logging/src/watcher/listener.rs#L4
