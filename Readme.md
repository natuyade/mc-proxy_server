>### ProxyServer for Minecraft
昔やりたかったMCServer用のProxyServerを作っています.

正直初見の知識ばっかりで苦戦してます(eguiやらTcpStreamやら)

---
### 対応表
- fileの保存場所切り替え(LocalAppData or Relative Path)
- 待ち受けパスの設定
- 許可するclientのServerAddressと接続するBackendAddressの指定
- logの保存(window正常終了時)
- allowしていないServerAddressはスルー
---
### まだ対応してないもの
- ProtocolVersion775以降,又は774未満のバージョン
  - 未検証なだけで使用可能なバージョンはいくつか存在する