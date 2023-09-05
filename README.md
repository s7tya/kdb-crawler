
# KdB Crawler

> ⚠️ KdB Crawler は安定しておらず、 JSON ファイルの形式は変更される可能性があります。

KDB の授業データを GitHub Actions でクロールして JSON 形式で提供する Rust 製ツールです。毎日、日本時間の0時にKdBからデータを取得し、差分があった場合にコミットを行います。

[kdb.json](https://raw.githubusercontent.com/s7tya/kdb-crawler/master/dist/kdb.json)
[更新履歴](https://github.com/search?q=repo%3As7tya%2Fkdb-crawler+%22Update+KdB+to%22&type=commits)

```ts
{
  "科目番号": string,
  "科目名": string,
  "授業方法": string,
  "単位数": string,
  "標準履修年次": string,
  "実施学期": string,
  "曜時限": string,
  "教室": string,
  "担当教員": string,
  "授業概要": string,
  "備考": string,
  "データ更新日": string
},
```