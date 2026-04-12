## ADDED Requirements

### Requirement: GameRegistry が inst ポインタを Key に MahjongWorld を管理する
The `GameRegistry` SHALL manage `MahjongWorld` instances using the `inst` pointer as a search key.
`GameRegistry` は `*mut c_void` 型の `inst` ポインタをキーとして、対応する `MahjongWorld` を非公開の `HashMap` で保持し、シングルトンの `static` として `Mutex` 経由でアクセスされなければなりません。

#### Scenario: inst を登録すると World にアクセスできる
- **WHEN** `registry.insert(inst, world)` を呼び出す
- **THEN** `registry.get(inst)` が `Some(&MahjongWorld)` を返す

#### Scenario: 未登録の inst に対して None が返る
- **WHEN** 登録していない `inst` で `registry.get(inst)` を呼び出す
- **THEN** 結果は `None` である

### Requirement: DLL の内部ポインタを Registry の外部に漏らさない
The `GameRegistry` SHALL NOT expose internal pointers of `MahjongWorld` to external DLLs.
`MahjongWorld` のアドレスや内部フィールドのポインタを DLL 側の `inst` メモリ領域に書き込んではならず、データの授受はコールバック関数を介した値のコピーのみで行われなければなりません。

#### Scenario: MJMI_GETTEHAI コールバックで DLL メモリにコピーが行われる
- **WHEN** `mjsend_message(inst, MJMI_GETTEHAI, ...)` が呼ばれる
- **THEN** Registry から取得した `Hand` コンポーネントの内容が `param2` アドレスの `MJITehai` 構造体へコピーされ、`MahjongWorld` のポインタは渡されない

### Requirement: Registry は複数の inst（マルチ対局）を同時サポートする
The `GameRegistry` SHALL support multiple `inst` pointers simultaneously for multi-game instances.
複数の異なる `inst` が同時に `GameRegistry` に登録された場合、それらが独立した `MahjongWorld` を参照することを保証しなければなりません。

#### Scenario: 2つの対局が互いのデータを干渉しない
- **WHEN** 2つの異なる `inst` が Registry に登録されている状態で、それぞれの `MJMI_GETTEHAI` が呼ばれる
- **THEN** 各呼び出しが自身の `MahjongWorld` の手牌を返し、もう一方の World には影響しない
