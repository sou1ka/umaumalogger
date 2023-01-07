# UmaUmaLogger うまうまロガー

DMM版「ウマ娘　プリティダービー」の育成内容を保存します。  
育成中のスピード、スタミナ、パワー、根性、賢さ、スキルPtを、画像認識（OCR）して数値を取得します。  
取得した情報はインストールフォルダの out フォルダに保存します。  

ソフトを起動して、 Start ボタンをクリックするとロギングを始めます。  
育成中断などでログの途中から追記したい場合は、対象のファイル名を入力して「START」ボタンをクリックしてください。  
定期的にウマ娘の画面をキャプチャし、各種ステータスをOCRで読み取ります。  
定期実行なので、タイミングによっては数値を取得できないタイミングが発生します。取得できない場合は、画面を少し止めておいてください。  

![育成中画面](http://www.plasmasphere.net/archives/umaumalogger/img/20221128235041.png)
![育成中画面](http://www.plasmasphere.net/archives/umaumalogger/img/20221223154632.png)  

「育成完了確認」の画面で、ステータスを数値に切り替えると、ログ上では「育成完了」になります。  

![育成完了画面](http://www.plasmasphere.net/archives/umaumalogger/img/20221130213826.png)  

※シナリオ事の特殊ステータス（パフォーマンスの値やショップポイントなど）は取得しません。  

「STOP」ボタンで任意のタイミングで停止できます。  
途中から再開する場合は、追記したいログファイル名を記入して「START」すると、追記モードでロギングを開始できます。  

OCRとフォントの都合で、3を8、8を3に間違える場合、連続する1を認識しない場合があります。  
気になる場合は、育成完了後に修正してください。  

### ログリスト

Listタブをクリックすると、ログファイル一覧が表示されます。  
ログファイルをクリックすると、グラフが表示されます。  
グラフは右クリックで保存できます。  

### イベントチェッカー

育成イベントの選択肢の結果を表示しています。（v0.5.0から）  
いわゆるイベントチェッカーです。  
育成ログをロギング中はイベント名を取得して自動で表示されます。  
イベントの結果が表示されないときは、イベント名を入力して「CHECK」ボタンを押すとチェッカー処理が走ります。  
ボタンを押しても表示されない場合は、検索が正常に動いていない可能性があります。  
結果の取得にインターネットへの通信が入りますので、ファイアーウォールの許可等は適宜行ってください。  

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

レジストリ等は使っていませんので、umaumaloger フォルダごと削除してください。  
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