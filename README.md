## adview

Adview: **A**nn**d**ata **View**er: head/less/view adata(h5ad) files in yout terminal.

### Why adview

Are you still doing this?:

```bash
❯ python3
Python 3.13.2 (main, Feb  4 2025, 14:51:09) [Clang 16.0.0 (clang-1600.0.26.6)] on darwin
Type "help", "copyright", "credits" or "license" for more information.
>>> import scanpy as sc   ## hold on, be patient with your HPC🚬
>>> adata = sc.read_h5ad('path/to/adata.h5ad')
>>> adata.var
>>> adata.obs
>>> adata.shape
```

I just want to glance！👀

Now, let `adview` comfort you!

### Installation

```bash
git clone https://github.com/JianYang-Lab/adview.git
cd adview
cargo build --release
./target/release/adview -h
```

or just

```bash
cargo install --git https://github.com/JianYang-Lab/adview.git
adview -h
```

### Usage

```bash
❯ adview -h
adview -- Adata Viewer: Head/Less/Shape h5ad file in terminal

Version: 0.1.0

Authors: wenjiewei<weiwenjie@westlake.edu.cn>

Usage: adview <COMMAND>

Commands:
  obs-head    Show first n obs [aliases: oh]
  obs-all     Show all obs [aliases: oa]
  var-head    Show first n var [aliases: vh]
  var-all     Show all var [aliases: va]
  shape       Show shapes of obs and var [aliases: s]
  export-obs  Export obs data to CSV file [aliases: e]
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Example

```bash
❯ adview oh -n 5 path/to/adata.h5ad
1: AAACCCAAGACTTCGT
2: AAACCCAAGCCTTTGA
3: AAACCCAAGTATGAAC
4: AAACCCAAGTCCGTCG
5: AAACCCAAGTGCAACG

❯ adview vh -n 5 path/to/adata.h5ad
1: ENSG00000243485
2: ENSG00000237613
3: ENSG00000186092
4: ENSG00000238009
5: ENSG00000239945

❯ adview s path/to/adata.h5ad

obs shape: 15235
var shape: 36601

❯ adview e path/to/adata.h5ad|less -S
```

### Contribution

code: [wenjiewei](https://github.com/wjwei-handsome)

inspiration: [liyang](https://github.com/LeonSong1995),[lounan](https://github.com/SGGb0nd),[wenhao](https://github.com/Ganten-Hornby),[dingyi](https://github.com/dingyigithub)

### License

MIT
