<script>
    import { invoke } from "@tauri-apps/api/tauri";
    import { Command } from '@tauri-apps/api/shell';
    import { convertFileSrc } from '@tauri-apps/api/tauri';

    import Button, { Group, Label, Icon } from '@smui/button';
    import Tooltip, { Wrapper } from '@smui/tooltip';
    import LayoutGrid from '@smui/layout-grid';
    import Cell from '@smui/data-table';

    let scid;

    async function take_screenshot() {
        let now = new Date();
        let filename = String(now.getFullYear()) + String(now.getMonth()+1).padStart(2, '0') + String(now.getDate()).padStart(2, '0') + String(now.getHours()).padStart(2, '0') + String(now.getMinutes()).padStart(2, '0') + String(now.getSeconds()).padStart(2, '0') + ".png"
        let path = await invoke("get_path");
        let cmd = new Command('capture', ["umamusume", path + "\\screenshot\\" + filename]);
        cmd.spawn();

        scid = setTimeout(get_imagelist, 5000);
    }

    async function imageview(img) {
        let path = await invoke("get_path");
        let cmd = new Command('view', ["/c", "start", path + "\\screenshot\\" + img]);
        cmd.spawn();
    }

    let imagelist = [];
    async function get_imagelist(force) {
        //if(force) { imagelist = []; }
        let ret = await invoke("get_imagelist");
        let images = JSON.parse(ret);

        if(true) {
            imagelist = [];
            imagelist = images;
        } else if(imagelist.length < images.length) {
            imagelist.push(images[images.length-1]);
        }

        clearTimeout(scid);
    }
    get_imagelist(true);

    async function show_screenshotdir() {
        let path = await invoke("get_path");
        let cmd = new Command('view', ["/c", "start", "explorer", path + "\\screenshot\\"]);
        cmd.spawn();
    }
</script>

<Group variant="raised">
    <Wrapper>
        <Button on:click={take_screenshot} variant="raised">
        <Icon class="fa-solid fa-image"></Icon>
        <Label>Capture</Label>
        </Button>
        <Tooltip>スクリーンショットを撮ります</Tooltip>
    </Wrapper>
    <Wrapper>
        <Button on:click={show_screenshotdir} variant="raised">
        <Label>Folder Open</Label>
        <Icon class="fa-regular fa-folder-open"></Icon>
        </Button>
        <Tooltip>スクリーンショットフォルダを開きます</Tooltip>
    </Wrapper>
</Group>

<hr />
<div class="screenshots">
    <LayoutGrid>
    {#each imagelist as img, i}
        <Cell on:click={imageview(img.filename)}>
            <img src={convertFileSrc("screenshot/" + img.filename) } width="90" />
        </Cell>
    {/each}
    </LayoutGrid>
</div>