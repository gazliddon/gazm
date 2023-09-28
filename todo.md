# Nice
* https://github.com/lukaslueg/built/tree/master/example_project
* https://github.com/windwp/nvim-autopairs

# Namespace importing
Writing to symbols


# Full scope symbol resolution
* Overtime replace stargate EQ4 equates with direct references to scoped label
* start with SLEEP
    * remove from EQ4
    * chnage all references to SLEEP to ::background::SLEEP


# Parsing
* Add in scope operators for symbol Parsing
* A symbol may have a path and that path might be absolute or relative

# Todo Scoping
* Properly scope local labels
    * Local labels are assigned unique label text rather than using the scope system
    * And then converted to normal scoped labels
    * Instead they should remaiing as local labels
    * And ONLY search the current scope rather than up through the scope chain
* Scope macros
    * Macro defs are currently unscoped
* Add barriers for scope resolution
    * Resolution of a symbol at current scope traverse up the scope chain to find the symbol
    * That's fine at file resolution
    * But ultimately I want each file to only import the symbols it needs to see
    * So a scope should be able to terminate any searches of its parents
* Separate scope navigation from the symbol table
    * Remove current scope tracking from the symbol table
    * Put in it's own object
    * Have the idea of a scope stack
    * As well as direct navigation
    * Can be read-only? - Navigation by ScopeNavigator - writing by symbols object

# File position to AST node mapping
* Make this generic in some way?
    * We only want certain nodes mapped?

## Require
- [ ] Change scope syntax to be in a block?

## Misc
- [ ] Text position -> Ast Item
- [x] Move FMT in gazm
- [x] Make all line counts start from line 0, only > 1 for displaying etc

## LSP
* Allow for multiple project file
* Get document changes
    - [*] Initial implementation register only for complete doc changes
    - [*] Add interface to incorporate change of entire doc
    - [ ] Longer term add in local changes for doc
    - [ ] Send back errors

* Inline errors
    - [*] Recompile on doc open
    - [*] Recompile on doc change
    - [ ] Recompile work space changes

* [*] Show references
* Show symbol value
    * On Shift-K on references
    * Next to definitions

# Fix
* Return a proper range for goto defintion

# Async Stuff
* In loop
    * Get file_name
    * expand to full path
    * If tokens in cache then ok
    * If not
        * Read the file
        * Queue (parent, file_name, file_id, file_text, opts)

* In Async
    * Turn into tokens
    * Return Tokenized result


