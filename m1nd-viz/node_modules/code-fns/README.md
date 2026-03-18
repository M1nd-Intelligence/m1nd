# code-fns

A utility library for visualizing code.

## Install

```bash
npm install code-fns
```

## Purpose

Most code highlighters in JavaScript output HTML and CSS, but if your intended
target isn't a web page, the tags and styles would then need to be translated
to the desired form. `code-fns` outputs raw text and hex colors, making it easy
to render the code in whichever form you choose. Specifically, `code-fns` was
built for use in the Motion Canvas project, for visualizing code in videos and
animations. `code-fns` may also compute the transformation between different
code blocks, so that you may animate between them.

## Compatibility

Supports all browsers and all [maintained node
versions](https://github.com/nodejs/Release), though you will need to use your
own transpiler, as the package files use modern EcmaScript features. This
ensures that you may configure your build as you wish.

## Usage

To parse code into highlighted tokens, use `language.lang-name` to select your
language, and `parse` to highlight it.

```tsx
import { ready, language, parse } from './tags';

await ready();
const tsx = language.tsx;
parse(tsx`() => true`);
```

This will generate the following output.

```tsx
[
  { code: '() ', color: '#c9d1d9' },
  { code: '=>', color: '#ff7b72' },
  { code: ' ', color: '#c9d1d9' },
  { code: 'true', color: '#79c0ff' },
];
```

You may then use templating to generate your code dynamically.

```tsx
import { ready, language, parse } from './tags';

await ready();
const tsx = language.tsx;
const generate = (result: string) => tsx`(${result});`;
parse(generate('false'));
```

```tsx
[
  { code: '(', color: '#c9d1d9' },
  { code: 'false', color: '#79c0ff' },
  { code: ');', color: '#c9d1d9' },
];
```

To compute the difference between two generated chunks of code, use `diff`.

```tsx
import { ready, language, diff } from './tags';

await ready();
const tsx = language.tsx;
const generate = (result: string) => tsx`(${result});`;
diff(generate('true'), generate('false'));
```

```tsx
[
  { code: '(', color: '#c9d1d9', morph: 'retain' },
  { code: 'true', color: '#79c0ff', morph: 'delete' },
  { code: 'false', color: '#79c0ff', morph: 'create' },
  { code: ');', color: '#c9d1d9', morph: 'retain' },
];
```

This can be helpful to create transitional animations between code, as in
Motion Canvas.
