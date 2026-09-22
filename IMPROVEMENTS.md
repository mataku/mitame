# mitame 改善点メモ

コードとドキュメントを読んだうえでの評価と、改善候補の一覧。

## 総評

一部の条件では他ツールより明確に有利。ただし差別化できている幅はまだ狭く、「Flutter の golden を SDK 更新のたびに作り直す手間を減らすこと」と「複数プラットフォームを同じ運用で回せること」の2点に限られる。差分アルゴリズム（pixelmatch 由来の YIQ 色差とアンチエイリアス検出）は reg-cli などにもあるため、独自の強みにはならない。

## 優位性

- **撮影と比較を分けている**：アダプタは PNG と sidecar（メタデータの JSON）を書き出すだけで、依存はプラットフォーム SDK のみ。Roborazzi や Paparazzi のように、Gradle/AGP の更新に巻き込まれて壊れることが起きにくい。
- **3プラットフォームで仕組みが共通**：同じ `mitame.toml`、`result.json` のスキーマ、HTML レポート、CI 手順を Flutter・Android・iOS で使える。方向性の近い reg-suit には撮影用のアダプタがなく、Node 環境と S3/GCS が前提になる。mitame は git だけで完結する。
- **既定値が厳しめで、その根拠を実測で示している**：比率のしきい値はデフォルト 0 で、ノイズは色の許容差とアンチエイリアス検出で吸収する。1単語の変更（1728 px）を見逃さないことを数値で確認している。
- **Flutter で撮影経路を変えずに導入できる**：既存の `matchesGoldenFile` をそのまま使え、差分があるときのテスト時間は stock の約1/5（`bench/README.md`）。
- **エージェントから扱いやすい**：`diff_bounds`、JSON Schema、`skills/` があり、AI による差分レビューを前提にした設計。

## 他ツールに劣る・並んでいるだけの点

| 観点 | 状況 |
|---|---|
| Ahem フォントで macOS と Linux の baseline を1つにする | alchemist の CI golden と同じ発想で、独自性はない |
| Android | Roborazzi は HTML レポート、compare モード、Compose Preview の自動撮影まで持っており、機能面では劣る |
| iOS | swift-snapshot-testing の `perceptualPrecision` や多彩な撮影方式（strategy）に対し、撮影面が薄い |
| SaaS（Chromatic、Emerge など） | 承認フロー、PR 上での画像表示、履歴管理がない |

## 現状の問題点

1. **README の Status 節が古い**：「The adapters are not yet published as packages」とあるが、実際は pub.dev と Maven Central に公開済み。
2. **Flutter でテストディレクトリが `test/` 固定**：`adapters/flutter/lib/src/mitame.dart` で `Directory.current/test` を基準にしており、`integration_test/` やモノレポのサブディレクトリから実行すると `StateError` になる。
3. **iOS の撮影方式に弱点がある**：`layer.render(in:)` を使っているため、`UIVisualEffectView`、一部の SwiftUI 描画、Metal を使う View が正しく描かれない。また、非推奨の `UIScreen.main.scale` を使っている。
4. **iOS だけ `MITAME_OUTPUT_DIR` が必須**：Flutter と Android にはデフォルトの出力先（`<cwd>/.mitame/current`）があり、挙動がそろっていない。
5. **baseline の差分にノイズが出る**：`captured_at` が sidecar に入るため、`--update` のたびに JSON も書き換わる（roadmap に記載済みで未解決）。
6. **動的な領域を除外する手段がない**：時刻やアニメーションなどの部分をマスクする設定がない。実アプリでは早い段階で必要になる。
7. **テストが手薄**：
   - アダプタ（Dart、Kotlin、Swift）側に単体テストがない。
   - `compare.rs` と `update.rs` の単体テストが 0 件（`tests/e2e.rs` のみ）。
   - AGENTS.md は「3つの正規化実装を同じ結果に保つ」ことを求めているが、それを検証するテストがない。
8. **CI からローカルへの baseline 更新経路がない**：CI 専用プロファイル（Linux 用の real フォント baseline など）をローカルに反映する手段がない（`compare --update --from` は未実装）。
9. **sidecar の読み込みエラーの扱いが一貫していない**：`compare_one` では current の sidecar が壊れていても無視して処理を続けるが、`compare_pair` ではエラーにしている。また、サイズ不一致の判定が sidecar のスキーマチェックより先に行われるため、スキーマ不一致なのに `mismatch` として報告されることがある。

## 改善提案（優先度順）

1. **README の Status を修正する**：すぐ直せる。
2. **Flutter の `testRoot` を設定可能にするか、`LocalFileComparator.basedir` から推定する**：導入時につまずく原因を取り除く。
3. **マスク・除外領域を追加する**：`[[rules]]` に `ignore = [{ x, y, width, height }]` を追加し、binary 側で処理する。AGENTS.md の「新しい挙動はアダプタの入力ではなく `mitame.toml` で設定する」という方針にも合う。
4. **`captured_at` を baseline に入れない**：`--update` 時に除去するか、current 側だけに持たせる。
5. **identity 正規化の共通テストケースを JSON で用意する**：3アダプタと Rust で同じケースを流し、実装のずれを防ぐ。
6. **iOS で `drawHierarchy(in:afterScreenUpdates:)` を選べるようにする**：撮影の正確さを上げる。スケールは `UITraitCollection.current.displayScale` から取る。
7. **iOS にもデフォルトの出力先を持たせる**：シミュレータから見えるホスト側パスの扱いは要検討。
8. **`compare --update --from <dir>` を実装する**：roadmap 既載。CI とローカルの往復が楽になる。
9. **`mitame doctor` を実装する**：roadmap 既載。導入時の設定ミスを検出する。
10. **Compose Preview や Flutter の `@Preview` から自動で撮影する**：Roborazzi との機能差を埋める最大の要素。

## 方針

「Flutter の golden 運用を楽にする」ことを主軸にし、ネイティブ側は「共通の CI とレポート基盤」として割り切るのが妥当。README の「Who it is for」もこの整理になっているので、方向性は合っている。

## 改善をすべて実施した場合の評価

全部やると、「条件次第で選ぶ価値がある」から「Flutter なら第一候補、ネイティブでも複数プラットフォーム運用なら有力」くらいまで上がる。ただし、改善点の多くは導入時のつまずきや品質の穴を埋めるもので、他ツールとの力関係を変えるのはマスク機能（#3）と Preview からの自動撮影（#10）の2つだけ。

### 項目ごとの効き方

| 分類 | 項目 | 評価への影響 |
|---|---|---|
| 必須機能の補完 | マスク・除外領域（#3） | **大**。これがないと実アプリでは採用しにくいので、採用候補に入る前提条件が満たされる |
| 他ツールとの差を埋める | Preview からの自動撮影（#10） | **大**。Roborazzi が選ばれる最大の理由がなくなり、「テストを書かずに VRT できる」段階に並ぶ |
| 導入時のつまずき解消 | README 修正、`testRoot` 設定、iOS の出力先デフォルト、`doctor` | 中。評価は変わらないが、試して離脱する人が減る |
| 日常運用の改善 | `captured_at` の除去、`--from` | 中。baseline の差分がきれいになり、CI 専用プロファイルも運用できるようになる |
| 信頼性の向上 | テスト拡充、正規化の共通テストケース、sidecar エラーの統一、iOS の `drawHierarchy` | 小〜中。外から見えにくいが、1.0 を名乗るための土台になる |

### プラットフォーム別の立ち位置

- **Flutter**：OSS の中では最有力になる。stock の golden と alchemist に比べ、tolerance、レポート、`--update` の運用、速度で明確に上回る。
- **Android**：Roborazzi とほぼ同等になる。依存の少なさと他プラットフォームとの共通運用で上回り、エコシステムや利用実績では劣る。単独の Android プロジェクトでわざわざ乗り換える理由は弱いまま。
- **iOS**：swift-snapshot-testing の撮影方式の豊富さには届かない。「他のプラットフォームと同じ運用で回したい」場合に選ばれるツール、という位置づけは変わらない。

### 改善点をすべてやっても残る課題

- **実機・エミュレータでの撮影がない**：Robolectric やシミュレータでの View 単位の撮影に限られるため、画面全体の VRT は Emerge や Maestro 系の領域のまま。
- **レビュー体験**：PR 上に画像を直接出せず、承認フローもない。SaaS に対しては「無料でサービスに依存しない」という価値だけで勝負することになる。
- **baseline を git に置く方式の規模限界**：数千枚規模になると LFS の運用が前提になり、ドキュメントでの案内も必要。
- **実績とコミュニティ**：コードでは解決できない部分で、採用判断では実はここが一番効く。

### 優先順位

マスク機能 → Flutter の `@Preview` 対応 → 実プロジェクトでの導入事例を記事にする、の順が評価を上げる近道。

## CLI の配布方法：ランチャー方式

今は CLI を brew か curl で入れ、アダプタも追加するという2ステップが必要になる。また、CLI はマシン単位でインストールされるのに、アダプタはプロジェクト単位で固定されるため、バージョンがずれうる（`schema_version` のチェックはこのずれへの対処）。

これを解消するため、esbuild や Biome が npm でやっているのと同じ方式をとる。バイナリは今のまま1つ作り、各エコシステムのパッケージからそのバイナリを取得して起動する。

| プラットフォーム | 起動方法 | バイナリの入手方法 |
|---|---|---|
| Flutter | `dart run mitame_flutter:mitame run` | 初回実行時に、パッケージのバージョンに対応する GitHub Releases のバイナリを取得し、チェックサムを検証してキャッシュする |
| Android | Gradle プラグインの `./gradlew mitameRun` | protoc と同じく、Maven Central の OS 別アーティファクト（classifier 付き）から取得する |
| iOS | SwiftPM の command plugin で `swift package mitame run` | SwiftLint と同じく、artifactbundle の `binaryTarget` として配る |

- **導入が1ステップになる**：別途インストールが不要になり、バージョンはプロジェクトの lockfile で固定されるので、CLI とアダプタのずれがなくなる。
- **速度は今のまま**：比較は今と同じくテストの外で、全体に対して1回行われる。
- **アダプタの依存方針も崩れない**：ランチャーは別パッケージ、またはアダプタとは別のエントリポイントとして用意する。
- **brew での導入も残せる**：複数リポジトリで同じバイナリを使いたい人は、これまで通り brew を使える。

### FFI で各アダプタに Rust の処理を同梱しない理由

- **テスト全体の終了後に動く共通の仕組みがない**：レポート生成や `--update` は全テスト分の結果がそろってから動く必要があるが、`flutter test` はテストファイルごとに別プロセスで動き、全体の終了時に呼ばれるフックがない。結局、結果を集約するためのコマンドが必要になり、CLI がなくならない。
- **配布の組み合わせが増える**：
  - Android（Robolectric）で必要なのは、Android 向けではなくホスト OS 向けのネイティブライブラリ。JNA（サードパーティ）か、JDK 22 以降が必要な FFM が要る。
  - iOS はシミュレータ向けのビルドを xcframework にまとめる必要がある。
  - 合計で「3種類のバインディング × 5前後のターゲット」を保守することになる。
- **AGENTS.md の方針に反する**：「アダプタはプラットフォーム SDK にだけ依存する」という原則を崩す。

### 進め方

利用者が最も多い想定の Flutter の `dart run` ランチャーから始めるのが、手間に対して効果が大きい。

## テスト内アサーションとの関係

テストの中で比較し、差分があればそのテストを失敗させるのは主流のやり方（Flutter の `matchesGoldenFile`、Paparazzi と Roborazzi の `verify`、swift-snapshot-testing の `assertSnapshot`）。mitame は README の「Review instead of assert」のとおり、Roborazzi の `compare` モードや reg-suit と同じく、後でまとめて判定する側をあえて選んでいる。

| | テスト内で判定（主流） | 後でまとめて判定（mitame） |
|---|---|---|
| 失敗の見え方 | IDE 上でそのテストが赤くなり、どのテストか一目で分かる | CLI の終了コードとレポートで分かる |
| 単体のテストを IDE で実行したとき | そのまま判定される | 判定されない（`mitame run` を通す必要がある） |
| SDK 更新時の数 px の差 | テストが大量に失敗する | tolerance で吸収され、レポートで確認できる |
| 速度 | テストごとにデコードと比較が走る | 全体で1回、Rust で並列に処理する |
| 全体の一覧性 | 失敗したテストのログがばらばらに出る | HTML レポート1枚にまとまる |

テスト内で判定するために FFI で Rust の処理を持ち込む必要はない。

- **速さの理由と矛盾する**：mitame が速いのは、比較をテストの外に出しているから。テスト内で比較すると、Rust で書いてもテストごとにデコードが走るので、その利点がほぼ消える。
- **CI では今の方式でも失敗になる**：差分があれば `mitame run` が終了コード 1 を返すので、CI の判定としてはすでにアサーションの役割を果たしている。

### 欠けているもの：IDE でテストを1つだけ実行したときの体験

IDE から単体のテストを実行すると、撮影はされるが判定されず、結果が分からない。FFI なしで次のように補える。

- **実行コマンドを案内する**：ドキュメントで `mitame run -- flutter test test/login_form_test.dart` のように、テストファイルを指定して実行する方法を示す。ランチャーができれば `dart run mitame_flutter:mitame run -- <path>` で済む。
- **失敗時の出力を改善する**：終了コード 1 のとき、差分のあった id と、それを撮ったテストを表示する。
  - 今の sidecar にあるのは、Flutter の `ext.flutter.test_dir` とグループ名（Android はクラス名、iOS はファイル名から作る）まで。
  - テストファイルのパスそのものは持っていないので、sidecar に項目を足し、3アダプタで同じように書き出す必要がある。項目の追加だけなら `schema_version` を上げる必要はない。
