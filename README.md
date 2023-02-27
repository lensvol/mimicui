# mimicui

[![Build status](https://github.com/lensvol/mimicui/actions/workflows/build_checks.yaml/badge.svg)](https://github.com/lensvol/mimicui/actions/workflows/build_checks.yaml)
[![Code Coverage](https://codecov.io/gh/lensvol/mimicui/branch/main/graph/badge.svg?token=9UQH8NT0RU)](https://codecov.io/gh/lensvol/mimicui)
[![License](https://img.shields.io/github/license/lensvol/mimicui)](https://github.com/lensvol/mimicui/blob/master/LICENSE)

Toy HTML-to-JS converter with both CLI and WASM frontends.

[**Try it here!**](https://lensvol.github.io/mimicui)

## Example output

```javascript 
function createMimic() {
    const root = document.createElement('div');

    const paragraph = document.createElement('p');
    paragraph.style.cssText = 'sexy';

    const text = document.createTextNode('Hello, world!');

    root.appendChild(paragraph);

    paragraph.appendChild(text);

    return root;
}
```

## Usage

* Convert HTML code stored in the HTML file: 
    ```shell
    mimicui <path>
    ```

* Convert HTML read from STDIN:
   ```shell
  curl https://test.host/1.html | mimicui -
    ```
  
## Development

### Compiling from source
```shell
cargo build --target release
```

### Compiling into WASM module
```shell
wasm-pack build --target web

```

Output will be put into the `pkg/` directory.
