# unbom

## about

This is the Rust port of my [unbom](https://github.com/ssg/unbom) tool in C#
which basically removes UTF-8 BOM markers from files safely. UTF-8 BOM markers
aren't useful, and can even cause problems with some tools that are not designed
to handle them. Only use if you have problems with your UTF-8 files of course.

## usage

Remove UTF-8 markers from all "txt" files in the current directory, and save 
the original in ".txt.bak" files:

```
unbom *.txt
```

Perform the same, but do not create a backup:

```
unbom -n *.txt
```

Remove UTF-8 

## challenges

I'm porting these as Rust exercises. It brings interesting challenges. Porting 
this tool made me tackle these issues:

- Parsing command-line arguments in a cross-platform tool requires you to be
  aware of Unix wildcard expansion. I knew that the distinction existed, but 
  I didn't have to think about this before, so my design process for CLI tools
  on Windows were pretty straightforward. There's no way to receive wildcards 
  from command-line arguments on Unix without a specialized syntax or user
  specifically surrounding the argument with double quotes. Actually,
  now I understand why Unix `find` tool is designed the way it is.

- Temporary file handling is subject to a few security issues. I haven't thought
  about it much on Windows as I've regarded these tools mostly for personal
  use. But, Rust's API made me consider on ways of making it more secure
  especially regarding atomic temporary file creation and permissions handling.

- Writing CLI code for Rust can be very verbose if you want to be explicit about
  error handling (and I think you should). Almost every line of code needs 
  handling the error case, reporting that to user and bailing out with a failure
  exit code. I experimented with that before, but I think I found a better 
  balance in this codebase by using `Result<>` return type on `main()` and 
  `.inspect_err()` to catch and report errors to the users. Combined with `?`
  operator, it becomes both expressive and lean. 

# license

MIT License