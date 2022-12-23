<script>
  import { invoke } from "@tauri-apps/api/tauri";
  import { Command } from '@tauri-apps/api/shell';
  import { listen, emit } from '@tauri-apps/api/event';

  import Textfield from '@smui/textfield';
  import Button, { Label, Icon } from '@smui/button';
  import Tooltip, { Wrapper } from '@smui/tooltip';
  import logs, { get_loglists } from './Logs.svelte';

  let loggingMsg = ["Start をクリックするとロギングを開始します"];
  let now = new Date();
  let filename = String(now.getFullYear()) + String(now.getMonth()+1).padStart(2, '0') + String(now.getDate()).padStart(2, '0') + String(now.getHours()).padStart(2, '0') + String(now.getMinutes()).padStart(2, '0') + String(now.getSeconds()).padStart(2, '0') + ".tsv"
  let process = null;
  let iid = false;
  let logno = "0";
  let startStats = '';
  let stopStats = "disabled";

  function addMsg(m) {
      if(m == '') { return; }

      let tmp = loggingMsg.concat();
      tmp.push(m);
      loggingMsg = tmp;

      let ti = setTimeout(function() {
      document.querySelector(".console li:last-child").scrollIntoView();
      clearTimeout(ti);
      }, 100);
  }

  async function startLog() {
      if(process) { return; }

      let path = await invoke("get_path");
      let cmd = new Command('umalog', [
          path + "\\out\\" + filename
      ]);
      process = await cmd.spawn();
      addMsg("ロギングを開始しました。プロセスは " + process.pid + " です");
      startStats = "disabled";
      stopStats = "";

      get_loglists();
      //  iid = setInterval(get_stdoutfile, 2000);

      listen('logrefresh', function(ret) {
          log_refresh(ret.payload);
      });
      emit('logcheck', { lognoStr: String(logno), filename: filename });
      iid = true;
  }

  async function get_stdoutfile() {
    let ret = await invoke("get_filelog_lastline", {
      lognoStr: String(logno),
      filename: filename
    });

    if(ret) {
      let parse = ret.split("\n");
      if(parse.length > 0) {
        logno = String(parse[0]);

        for(let i = 1, size = parse.length; i < size; i++) {
          addMsg(parse[i].replace("\t", ", "));
        }
      }
    }
  }

  async function log_refresh(ret) {
    if(ret) {
      let parse = ret.split("\n");
      if(parse.length > 0) {
        logno = String(parse[0]);

        for(let i = 1, size = parse.length; i < size; i++) {
          addMsg(parse[i].replace("\t", ", "));
          if(parse[i].indexOf('育成完了') !== -1) {
            stopLog();
          }
        }
      }
    }

    if(iid) {
      emit('logcheck', { lognoStr: String(logno), filename: filename });
    }
  }

  async function stopLog(silent) {
//    clearInterval(iid);
    iid = false;
    let cmd = new Command('taskkill', ["/im", "umalog.exe", "/f"]);
    process = await cmd.spawn();
    process = false;
    logno = "0";
    startStats = "";
    stopStats = "disabled";
    
    if(silent == true ) { return; }

    addMsg("ロギングを停止しました");
  }
  stopLog(true);

</script>

<div class="row">
<Textfield variant="outlined" bind:value={filename} label="Filename"></Textfield>
    <Wrapper>
    <Button on:click={startLog} variant="raised" disabled="{startStats}">
        <Icon class="fa-solid fa-play"></Icon>
        <Label>Start</Label>
    </Button>
    <Tooltip>ロギングを開始します</Tooltip>
    </Wrapper>
    <Wrapper>
    <Button on:click={stopLog} variant="raised" disabled="{stopStats}">
        <Label>Stop</Label>
        <Icon class="fa-solid fa-stop"></Icon>
    </Button>
    <Tooltip>ロギングを停止します</Tooltip>
</Wrapper>
</div>

<hr />

<div class="console">
<ul>
{#each loggingMsg as msg}
    <li>{msg}</li>
{/each}
</ul>
</div>