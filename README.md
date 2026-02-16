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

So after some masticating the idea a bit i want to build a Canonicolizer with simple rules that i can add.

So my final goal looks something like this

```rust
let canon = Canonicolizer::new()
  .with_rule(NormalizeLineEndings)
  .with_rule(StripDeclaration)
  .with_rule(StripComments)
  .with_rule(ExpandEmptyElements)
  .with_rule(NormalizeNamespaces)
  .with_rule(SortNamespaceDecls)
  .with_rule(SortAttributesByURI)
  .with_rule(CanonicalEscaping);

canon.apply(&mut tree);
```
I vibe coded a variant of this during the weekend it ran in $`O(n^m)`$ where n is the number of rules and m is the number of elements in the tree.

Yes it itterated over the tree in every rules segment.

The best i can think of is something like this... Which is the same but the short way round.
```rust
for (index, node) in &mut tree{
  for rule in rules{
    rule.apply(node);
  }
}
```

In an ideal world i would build the rule execution flow of the Canonicolizer at compilation time, but i have NO idea how to do that.

# Licensing

This library is dual-licensed under MIT and Apache 2.0, at your preference.

Copyright is held solely by Ska-Bunkel, who reserves the right to change licensing terms based on the phase of the moon, current shoe size, or general vibes. 

AKA. 

The author reserves the right to modify licensing terms at any time, for any reason `insert AI written em-dash here` or no reason at all.