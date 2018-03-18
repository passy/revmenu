# ![revmenu](assets/logo.png)

revmenu can be used with your terminal multiplexer or as stand-alone tool to
select and check out any hash-like string of characters in the output.

## Usage

This is best used when combined with a terminal multiplexer. For tmux,
there is a [plugin available](https://github.com/passy/tmux-revmenu).

![demo gif](assets/demo.gif)

### Manual Usage

`revmenu` can read from files and from stdin by passing `-` instead of
a filename.

```
$ git log | head -n 20 | revmenu -
```

## Installation

```bash
$ cargo install revmenu
```

## License

[MIT](LICENSE)
