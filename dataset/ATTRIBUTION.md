# Wiki dataset attribution

`wiki.json` derives from the text of **The Binding of Isaac: Rebirth Wiki**
(https://bindingofisaacrebirth.wiki.gg), published under a **Creative Commons
Attribution-ShareAlike 4.0** license (https://creativecommons.org/licenses/by-sa/4.0/).
The derived dataset is distributed under the same license. The snapshot date and the
highest revision are in the file's `meta` field; for each entry, `revid` identifies the
page revision. Wiki images are not included.

A few descriptions are not the wiki's: where a page has none, or one that says nothing, the
project writes its own in `corrections.json` (`descriptions`), and those replace the wiki's
line in `wiki.json`. They are released under the same license.
