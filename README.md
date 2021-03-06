Rust のソースコードをバンドルします。

主な機能はこのあたりでしょうか。詳しくはこの README の下の方をご覧いただけるとです。
- モジュールのインライン展開
- テストモジュール、doc commentes の削除
- 外部クレート（`path` で指定されたもののみ、かつマクロ非対応）の展開


### 使い方

このレポジトリのルートを current directory にして、これです。（crates.io に公開しておりません。）

```bash
$ cargo install --path .
```

消したいときにはこうです。

```bash
$ cargo uninstall procon-bundler
```

使い方は 2 通りあります。一つはこのように、current directory から**クレートルートへのパスを指定します。** これ単体で試したいときにはこちらの方法がおすすめです。

```bash
$ procon-bundler --path path/to/your/library/crate/root
```

もう一つはこのように、current directory から**ワークスペースへのパスと、パッケージのお名前を指定します。** 決まったワークスペースからいろいろなクレートを取ってきたいときにおすすめです。他ツールと連携するときはこちらでしょうか。本ツールの Vim ラッパーである[ac-adapter-rs-vim](https://github.com/ngtkana/ac-adapter-rs-vim) はこちらを呼んでいます。

```bash
$ procon-bundler --repo path/to/your/library/workspace/root name_of_your_package
```

いずれにしても、**標準出力にバンドル結果が出力されます。**


### バンドラさんの仕様

#### バンドラさんのお仕事

まずは `Cargo.toml` を読んで、`dependencies` を把握します。

そしてソースコードを読んで行き、次の一連の操作 X をします。

操作 X:
- `cfg(test)` アトリビュートのついたインラインモジュールを除去します。
- インラインでないモジュールはパスを見て中身を見に行き、この一連の操作 X を適用してから**インラインに展開します。**

これが終わったら、
- `allow(dead_code)` アトリビュートをつけます。
- Vim 向けのフォールドマーカーをつけます。
- Doc comments を削除します。
- 外部クレートを参照するパス（ただしマクロは除く）を書き換えます。


#### 非対応の機能 〜 バンドラさんに気持ちよく働いていただくために

- rust-fmt でフォーマットされていないもの
- `macro_export`
- インラインモジュール以外の、`cfg(test)` アイテムの除去


#### ごめんなさい

あまり皆様につかっていただくつもりで作っていなかったのもあり、タブが 4 spaces だったり、Vim のフォールドマーカー（まあでも Vim は世界共通語のようなものではあり……です。）をつけたりなど、好み全開の機能がハードコーディングされています。**もしご要望があれば、多少はカスタマイズできるようにしてもよいかもしれません**ね。

意味がわかりやすいように、あたかも構文木に何かをするように書いていますが、実際には正規表現ゴリ押しですから、書いているように動かない可能性もあります。私の書くコードに対しては、もちろんうまく動くのですが、**Hack されると一溜りもない感じなので**お手柔らかにお願いできるとです……

あと、CI はおろかテストさえありませんから、気が向いたらなんとかしたいですね。

