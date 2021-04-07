# Uniphy
A simple web based tool to convert [Uniprot](http://uniprot.org) IDs into [Phytozome](https://phytozome.jgi.doe.gov/pz/portal.html) IDs and vice versa, written in Rust.

While working on my undergraduate level 3 project I found myself needing a method of converting a bunch of uniprot IDs to Phytozome transcripts a number of times, leading to the creation of this simple tool. I'm also aware it is terribly named.

Creates a simple webservice at localhost on port 3000. Releases available for Linux and Windows. There are no MacOS binaries but it should compile and run on MacOS.

It probably does not scale well, so do not expect lots of users using the service at once. It is also certainly not the best way to design such a tool, and the code is a little 'hackish' in places but it gets the job done.

**DISCLAIMER: No warranty is provided with this tool.**

**I will not be held responsible for any damage or loss of work.**

**Use at your own risk.**

Expectations:
* A downloaded Uniprot proteome of species X in FASTA format named 'uniprot_db.fasta' in the same folder as the executable
* A downloaded Phytozome proteome of species X in FASTA format named 'phytozome_db.fasta' in the same folder as the executable
* A list of identifiers you wish to convert, seperated by spaces, new lines or commas in the text field

## TODO:
* Don't hardcode the port, make this user configurable
* Enable multiple uniprot and phytozome proteomes to be used at once, possible discover proteomes present in a folder and allow selection with a dropdown menu?
