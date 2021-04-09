# Uniphy
A simple web based tool to convert [Uniprot](http://uniprot.org) IDs into [Phytozome](https://phytozome.jgi.doe.gov/pz/portal.html) IDs and vice versa, written in Rust.

While working on my undergraduate level 3 project I found myself needing a method of converting a bunch of uniprot IDs to Phytozome transcripts a number of times, leading to the creation of this simple tool.

Creates a simple webservice at localhost on port 3000. Releases available for Linux, Mac (untested) and Windows.

It probably does not scale well, so do not expect lots of users using the service at once. It is also certainly not the best way to design such a tool, and the code is a little 'hackish' in places but it gets the job done.

**DISCLAIMER: No warranty is provided with this tool.**  
**I will not be held responsible for any damage or loss of work.**  
**Use at your own risk.**

## Instructions:
1. Download a Uniprot proteome from [https://www.uniprot.org/proteomes](https://www.uniprot.org/proteomes/) of species X in FASTA format into the same folder as the uniphy executable and rename it to 'uniprot_db.fasta'.
2. Download a Phytozome proteome from [https://phytozome.jgi.doe.gov/pz/portal.html](https://phytozome.jgi.doe.gov/pz/portal.html) (requires an account) of species X in FASTA format into the same folder as the uniphy executable and rename it to 'phytozome_db.fasta'.
3. Run the executable by double clicking on the executable on windows or by entering "./uniphy" without the quotes from the command line in the downloaded folder in Linux or MacOS.
4. Navigate to http://localhost:3000 if a new browser window does not automatically start.

## TODO:
* Don't hardcode the port, make this user configurable
* Enable multiple uniprot and phytozome proteomes to be used at once, possible discover proteomes present in a folder and allow selection with a dropdown menu?
