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
