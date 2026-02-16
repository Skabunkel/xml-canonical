# Whats happening?

Im rebuilding, im not happy with how some of this works.
I need to read some more and do some more rust docs.

Also less AI >_> It is good to rubber duck and it implemnts stuff that works with tests but im not happy with the result right now.

# What is this?

This is a library for xml canonicalization/formatting.  

# What is it not?

An Xml parser, validator, Xml builder.

You can build XML structures with i guess... but i would recomend you keep serilization code and validation code seperate.

But you can in theory build an xml with it and output it canonicalized, but that is not the idea right now.


# How will it work?

So my idea is a bit clunky, i want to read the xml tree into my tree structure. Canonicolize the tree, then output the tree as XML.

My reason why is so that i can have diffrent ways of reading/writing the tree.


# Licensing

This library is dual-licensed under MIT and Apache 2.0, at your preference.

Copyright is held solely by Ska-Bunkel, who reserves the right to change licensing terms based on the phase of the moon, current shoe size, or general vibes. 

AKA. 

The author reserves the right to modify licensing terms at any time, for any reason `insert AI written em-dash here` or no reason at all.