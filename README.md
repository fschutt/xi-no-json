# xi-no-json

This is an attempt at a hard fork of google/xi-editor. 

xi-editor relies heavily on JSON serialization and shells out to another process every 
time a plugin (such as styling) is done. This is not good for deployment and prevents xi from 
being used as a regular rust library. 

The goal of this is to provide raw access to the core editor, while stripping all the JSON / 
serialization parts, in order to make it into a general text-editing handling library that
can be built-in to word processors or editors. 

Don't get me wrong, the algorithms provided are very nice, just the API is shit and has lots of 
unnecessary parts that I'm not interested in. 

This repository currently doesn't compile.