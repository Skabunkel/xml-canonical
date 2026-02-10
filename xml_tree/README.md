# What is this?

This is an arcane implemetation of a tree, specifically a tree optimized for preorderd trees.
With simpler words, a flat tree structure that is read and processed left too right.

## Why this kind of tree?
Well XML, JSON and Yaml can be though of this type of tree.
I used to work in finance and read up on stuff like this. It very usefull for formatting and canonicalization.

simplest implementation is two lists, one contains the tokens and one contains the depth. 
A node is the index in the lists, both lists needs to have the same number of elements.

Example
```
<root>
  hello
  <e1/>
  <e2>
    world
  </e2>
  <e3></e3>
</root>
```

| index | token   | depth |
| ----- | ------- | ----- |
| 0     | root    | 1     |
| 1     | "hello" | 2     |
| 2     | e1      | 2     |
| 3     | e2      | 2     |
| 4     | "world" | 3     |
| 5     | e3      | 2     |


So the root node would be index 0  

## Drawbacks
Adding and removing elements in the middle of the tree is inefficient and requiers a bunch of inserts and/or slice operations.

I will probably have to implement some of that when it comes to DTD stuff

## Changes comming.



After som reading i think need to move the namespaces into the attribute portion of the tags; or atleast add them to each tag.

Why? It seems i have been mistaken about how namespaces work.

they can be defined and redefined as you are going down the tree, and namespaces are only visible in the subtree under their definition. 

- [ ] Move namespaces onto the tags
- [ ] Add methods to mutate the tree (Needed for canonicolization.)



