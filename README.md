# UmaUmaLogger うまうまロガー

DMM版「ウマ娘　プリティダービー」の育成内容を保存します。  
育成中のスピード、スタミナ、パワー、根性、賢さ、スキルPtを、画像認識（OCR）して数値を取得します。  
取得した情報はインストールフォルダの out フォルダに保存します。  

ソフトを起動して、 アプリ右上の「>」ボタンをクリックするとロギングを始めます。  
育成中断などでログの途中から追記したい場合は、対象のファイル名を入力して「>」ボタンをクリックしてください。  
定期的にウマ娘の画面をキャプチャし、各種ステータスをOCRで読み取ります。  
定期実行なので、タイミングによっては数値を取得できないタイミングが発生します。取得できない場合は、画面を少し止めておいてください。  
イベントチェッカーを自動取得にする場合も、「>」ボタンを押してください。
![育成中画面](http://www.plasmasphere.net/archives/umaumalogger/img/202303291748.png)

育成画面は赤枠で囲んだ箇所を取得しています。  
![育成中画面](http://www.plasmasphere.net/archives/umaumalogger/img/20221128235041.png)
![育成中画面](http://www.plasmasphere.net/archives/umaumalogger/img/20221223154632.png)  

「育成完了確認」の画面で、ステータスを数値に切り替えると、ログ上では「育成完了」になります。  

![育成完了画面](http://www.plasmasphere.net/archives/umaumalogger/img/20221130213826.png)  

※シナリオ事の特殊ステータス（パフォーマンスの値やショップポイントなど）は取得しません。  

#### 既知の問題点
OCRとフォントの都合で、3を8、8を3に間違える場合、連続する1を認識しない場合があります。  
ウマ娘のウィンドウサイズが小さいと文字認識が出来ない場合があります。イベント名やログ保存が出来ない場合は、ウィンドウサイズを最大にしてお試しください。  
イベント名やステータスの上に別のアプリケーションがある場合は値を取得できなくなります。

### ログリスト

「ログリスト」をクリックすると、ログファイル一覧が表示されます。  
ログファイルをクリックすると、グラフが表示されます。  

#### グラフの種類

以下のグラフを表示できます。

* 育成ステータスの遷移（上部）  
* ステータスのポーラーグラフ（右側上）
* 育成した「やる気」の回数※やる気ダウン回数含む（右側下）

### イベントチェッカー

育成イベントの選択肢の結果を表示しています。（v0.5.0から）  
いわゆるイベントチェッカーです。  
育成ログをロギング中はイベント名を取得して自動で表示されます。  
イベントの結果が表示されないときは、イベント名を入力して「CHECK」ボタンを押すとチェッカー処理が走ります。  
結果の取得にインターネットへの通信が入りますので、ファイアーウォールの許可等は適宜行ってください。  

### スクリーンショット

ウマ娘のスクリーンショットを撮影する機能がついています。  
アプリの右上にある撮影ボタンか、アプリ下段にある撮影ボタンを押すと撮影できます。  
撮ったスクリーンショットはサムネイル表示されます。  
※スクリーンショット数が多い場合、アプリ起動時の読み込みに時間がかかる場合があります。

-----------------------------------------------------------------------------

## ソフト構成

* umaumalogger/
* UmaUmaLogger.exe	アプリ本体
* umalog.exe			ロギング処理本体
* bin/				OCR処理バイナリ
* out/				ログ出力フォルダ
* screenshot/			スクリーンショット保存フォルダ

構成を変えると動かなくなるので、変更しないでください。

-----------------------------------------------------------------------------

## umalog.exe

コマンドライン版です。  
引数無しで起動すると、出力ファイルを新規で作成します。  
引数有りで起動すると、対象ファイルに追記していきます。（無い場合は作成します）  

> .\umalog.exe out\デジたそ育成_20221112.txt

-----------------------------------------------------------------------------

## アンインストール

レジストリ等は使っていませんので、umaumalogger フォルダごと削除してください。  
テンポラリ（%temp%）にOCR処理したファイルが残りますので、気になる方は削除してください。  
対象: %temp%\umalog\  

-----------------------------------------------------------------------------

一部の OCR 処理に Tesseract OCR v3系のバイナリを使用しています。  
https://sourceforge.net/projects/tesseract-js.mirror/files/v3.0.2/  

グラフ表示に Chartjs を使用しています。  
https://www.scichart.com/javascript-chart/  

アイコンは ICOOON MONO を使わせてもらっています。  
https://icooon-mono.com/  

スクリーンショット処理に screenshot-rs のソースを流用しています。  
https://github.com/robmikh/screenshot-rs  

-----------------------------------------------------------------------------

ソフトの作者は sou1ka (@sou1ka)です。  

Copyright(c) 2022 sou1ka (@sou1ka)  
This software is released under the MIT License, see LICENSE.  

## License
MIT