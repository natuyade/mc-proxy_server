>### ProxyServer for Minecraft(prot_version774)
昔やりたかったMCServer用のProxyServerを作っています.

正直初見の知識ばっかりで苦戦してます(eguiやらTcpStreamやら)

---
### 対応表
- fileの保存場所切り替え(LocalAppData or Relative Path)
- 待ち受けパスの設定
- 許可するclientのServerAddressと接続するBackendAddressの指定
---
### まだ対応してないもの
- logの保存
- 指定した(allowしていない)ServerAddressのブロック機能
- ProtocolVersion774以降,又は未満のバージョン
  - 未検証なだけで使用可能なバージョンはいくつか存在する