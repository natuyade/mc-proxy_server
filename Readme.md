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

---
### 妥協部分
現状config,logの保存場所はOptionPathとboolでの分岐という

雑な書き方にしているため,今後固定パスをUser側で設定する方向へ変えたい.

ですが別のprojectを進めたいため一旦こちらの更新は止まります

今回の更新でwindow側のunwrapは解決したため現状満足
(2026/04/06)
---