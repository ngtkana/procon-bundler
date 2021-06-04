# procon-bundler

これは、競技プログラミングのためのバンドルツールです。Rust のクレート全体を、別のクレートのトップレベルモジュールに貼り付けてブロクモジュールとして使えるような文字列に変換します。


## Insallation

このレポジトリをクローンして、cargo-install してください。今のところ、crate.io さんに publish する予定はありません。

```
> git clone git@github.com:ngtkana/procon-bundler.git
> cd procon-bundler
> cargo install --path procon-bundler
```

試したいだけならばインストールしなくても使えます。（Usage のセクションをご覧ください。）


## Usage

Installation にあるようにすると、`procon-bundler` コマンドがインストールされます。`bundle`, `find` のサブコマンドがあります。`bundle` はクレートへのファイルパスを指定して、それをバンドルします。`find` は、ワークスペースへのファイルパスとクレート名を指定して、そのクレートをバンドルします。


```
> procon-bundler bundle "${PATH_TO_THE_CRATE_ROOT}"
```

```
> procon-bundler find "${PATH_TO_THE_WORKSPACE_ROOT}" "${CRATE_NAME}"
```

インストールしていない場合は、カレントディレクトリをこのレポジトリにしてこれです。

```
> cargo run -- bundle "${PATH_TO_THE_CRATE_ROOT}"
```

```
> cargo run -- find "${PATH_TO_THE_WORKSPACE_ROOT}" "${CRATE_NAME}"
```



## Effects

このレポジトリ直下に、 [procon-bundler-sample](https://github.com/ngtkana/procon-bundler/tree/master/procon-bundler-sample/src)、[procon-bundler-sample-result](https://github.com/ngtkana/procon-bundler/blob/master/procon-bundler-sample-result/src/lib.rs)  があるのですが、前者をバンドルすると後者になることがテストで保証されています。


### features

* モジュールの展開とインデントの調整（インライン、ブロックともに）
* `cfg(test)` つきモジュールの消去（インライン、ブロックともに）（モジュール以外のアイテムは消去されません。）
* doc comments の消去（4 種類すべて）
* パスの置換（マクロ、非マクロともに）
* フォールドマーカー `{{{`, `}}}` の付加



## Example

私は普段、[ac-adapter-rs](https://github.com/ngtkana/ac-adapter-rs) というライブラリと、[ac-adapter-rs-vim](https://github.com/ngtkana/ac-adapter-rs-vim) というスクリプトを使っています。
