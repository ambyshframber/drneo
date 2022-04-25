# drneo

static site generator for neocities.

`drneo` is designed to make running a neocities site much more painless, allowing you to write page content in markdown instead of html, and letting you keep your entire site source locally. it walks through a site source directory and automatically processes all `.md` files into html. it then uploads these processed versions and all the other files to your neocities site.

## how to use

### directory structure

call `drneo` in a directory with the following structure (files marked with `*` are required):

- root
    - cfg
        - api_key *
        - md_ignore
        - md_postfix *
        - md_prefix *
        - md_replace
    - site
        - (site content here)

`cfg` contains configuration data for the program. the files are explained below:

- `api_key`: a neocities API key (run `curl "https://USER:PASS@neocities.org/api/key"` to get one for your account)
- `md_ignore`: a list of files to not process into html and instead upload as raw `.md` files
- `md_postfix`: a section of text to append to all markdown files
- `md_prefix`: a section of text to prepend to all markdown files. also supports `##EXTRAHEAD##` (more on that later)
- `md_replace`: a list of replacements to make in markdown files (more on that later)

`site` can contain anything you like. just remember that ALL `.md` files will be processed into html unless you specifically tell the program to ignore them.

### command line options

- `-d DIRECTORY`: run the program in `DIRECTORY`, not the cwd
- `-r REPLACEMENT`: add a replacement to the dictionary
- `-i FILE`: ignore `FILE` when processing markdown

## replacements and `##EXTRAHEAD##`

`drneo` supports replacements. these are pretty self explanatory. the format is as so: `TRIGGER=REPLACEMENT`, and they are inserted in text like so: `text REP=TRIGGER text`.
in this example, `REP=TRIGGER` will be replaced with `REPLACEMENT` in every file it occurs in.

the phrase `##EXTRAHEAD##` can also be inserted into your `md_prefix` file. any lines at the start of a markdown file that begin with `(HEAD)` will be added here, and removed from the markdown. this lets you provide per-page titles, styling and javascript, for example, by inserting them into the `<head>` tag of the html.

## why drneo?

simple. markdown is `md`, and "neocities" starts with "neo". "md" also means "doctor of medicine".
