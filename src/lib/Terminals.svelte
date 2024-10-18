<script >
    import { invoke } from '@tauri-apps/api';
    import { page } from '$app/stores';
    import { derived } from 'svelte/store';
    import { open } from '@tauri-apps/api/dialog';
    import { onMount } from 'svelte';
    import Devices from '../lib/Devices.svelte';
    import Progress from './Progress.svelte';
    import ProgressBig from './ProgressBig.svelte';
    import ProgressMain from './ProgressMain.svelte';
    export let TerminalSlave = [];
    export let TerminalMaster= [];
    export let stateDevises;
    export let ActCompanyName;

    let maxSizeMaster = false
    let maxSizeSlave = false
    let setMaster = true
    let setSalve = false
    
    let commandmaster = ''
    let commandslave = ''

    let fileMasterLic = "";
    let fileMasterSwk = ""; 
    let fileSlaveLic = "";
    let fileSlaveSwk = "";
    let progress = 0;
    let progressMain = 0
    
    let all_ip_master = [];
    let all_id_slave = [];
    let masterIp = "";
    let slaveIp = "";
    let rsPort = 873;
    let is_protectedRsPort = false;
    let open_master_ip = false;
    let open_svave_ip = false;
    let open_swimming_ip = true
    let open_pingGw = true;
    let pahtdownload = ""

    let is_install_master = false;
    let is_install_slave = false;

    let swimming_ip = "";
    let pingGw = "";
    let subnetMaskIdex = 0;

    let masterStep = 0;

    let terminalContainerMaster;
    let autoScrollMaster = true;

    let terminalContainerSlave;
    let autoScrollSlave = true;

    let installation_stage = [
        "CREATE",
        "SCAN",
        "ARCHIVEGENERATION",
        "ARCHIVEDOWNLOAD",
        "SENDINGFILES",
        "INSTALLATE",
        "CONFIGURATOR",
        "DBCONFIGURATOR",
        "COROPASE",
    ]

    let subnetMasks = [
        "",
        "255.255.255.255 (/32)",
        "255.255.255.254 (/31)",
        "255.255.255.252 (/30)",
        "255.255.255.248 (/29)",
        "255.255.255.240 (/28)",
        "255.255.255.224 (/27)",
        "255.255.255.192 (/26)",
        "255.255.255.128 (/25)",
        "255.255.255.0 (/24)",
        "255.255.254.0 (/23)",
        "255.255.252.0 (/22)",
        "255.255.248.0 (/21)",
        "255.255.240.0 (/20)",
        "255.255.224.0 (/19)",
        "255.255.192.0 (/18)",
        "255.255.128.0 (/17)",
        "255.255.0.0 (/16)",
        "255.254.0.0 (/15)",
        "255.252.0.0 (/14)",
        "255.248.0.0 (/13)",
        "255.240.0.0 (/12)",
        "255.224.0.0 (/11)",
        "255.192.0.0 (/10)",
        "255.128.0.0 (/9)",
        "255.0.0.0 (/7)",
        "254.0.0.0 (/6)",
        "252.0.0.0 (/5)",
        "248.0.0.0 (/4)",
        "240.0.0.0 (/3)",
        "224.0.0.0 (/2)",
        "192.0.0.0 (/1)",
        "128.0.0.0 (/0)"
    ];
    let installation_stage_master = 0;
    let installation_stage_slave = 0;

    async function selectFile(name, f) {
    try {
        const file = await open({
        multiple: false,
        filters: [{ name: name, extensions: [f] }]
        });
        

        if (file) {
        // Путь к файлу передается на бэкенд
        return file
        }
    } catch (error) {
        console.error('Error selecting file:', error);
    }
    }
    async function uploadFile(ismaster) {
        if (ismaster) {
            await invoke('upload_file', { licfilepath: fileMasterLic, swkfilepath:fileMasterSwk,  ismaster: true, })
            all_ip_master = await get_device_ip(true)
        } else {
            await invoke('upload_file', { licfilepath: fileSlaveLic, swkfilepath:fileSlaveSwk,  ismaster: false, })
            all_id_slave  = await get_device_ip(false)
        }
        progress += 1
        progressMain +=1
    }
    

    async function SelectSwkLic(ismaster) {
        if (ismaster) {
            if (fileMasterLic === "") {
                fileMasterLic = await selectFile('файл lic', 'lic');
            } else if (fileMasterSwk === ""){ 
                fileMasterSwk = await selectFile('файл swk', 'swk');
            }
        } else {
            if (fileSlaveLic === "") {
                fileSlaveLic = await selectFile('файл lic', 'lic');
            } else if (fileSlaveSwk === ""){ 
                fileSlaveSwk = await selectFile('файл swk', 'swk');
            }
        }
        progressMain +=1
    }
    async function delete_device(ismaster) {
        return await invoke('delete_device', { ismaster })
    }

    function showNotification(message) {
        const notificationElement = document.getElementById('notification');
        const notificationText = document.getElementById('notification-text');
        
        // Установить текст уведомления
        notificationText.textContent = message;
        
        // Показать уведомление
        notificationElement.classList.remove('hidden');
        notificationElement.classList.add('visible');
        
        // Скрыть уведомление через 10 секунд
        setTimeout(() => {
            notificationElement.classList.remove('visible');
            notificationElement.classList.add('hidden');
        }, 10000);
    }


    async function start_config(ismaster) {
        console.log("start config")
        await invoke('start_config_program', { masterip: masterIp, slaveip: slaveIp, swimmingip:swimming_ip, ismaster:ismaster })
        open_swimming_ip = false
        if (ismaster) {
            masterStep = 5
        }
        progress += 1
        progressMain += 5
    }

    async function start_config_db() {
        console.log("start config")
        await invoke('start_config_db', { masterip: masterIp, slaveip: slaveIp })
        progress += 4
        progressMain +=10
    }

    async function start_coropace() {
        open_master_ip = false;
        open_svave_ip = false;
        open_pingGw =false;
        await invoke('start_coropace', { masterip:masterIp, slaveip:slaveIp, swimmingip:swimming_ip, maskindex:subnetMaskIdex, pinggw:pingGw});
        open_swimming_ip = false
        progress += 1
        progressMain +=10
        
    }

    async function start_installate(ismaster) {
        is_protectedRsPort = true;
        open_master_ip = false;
        open_svave_ip = false;
        await invoke('start_install', { masterip: masterIp, slaveip: slaveIp, port: rsPort, ismaster:ismaster })
        if (ismaster) {
            is_install_master = true;
            masterStep = 4
        } else {
            is_install_slave = true;
        }
        progress += 1
        progressMain +=10
    }

    async function change_state(ismaster) {
        let index = 0;
        if (ismaster) {
            index = installation_stage_master
        } else {
            index = installation_stage_slave
        }
        await invoke('change_state', {ismaster, index});
        open_master_ip = true;
        open_svave_ip = true;
        is_install_master = true;
        is_install_slave = true;

        all_ip_master = await get_device_ip(true);
        all_id_slave = await get_device_ip(false);
    }
    async function get_device_ip(ismaster) {
        return await invoke('get_device_ip', { ismaster })
    }
    async function sendMaster() {
        await invoke('send_command', { ismaster: true, command: commandmaster+'\n' })
    }

    async function signalMaster() {
        await invoke('send_signal', { ismaster: true, signal: "INT" })
    }
    async function sendSlave() {
        await invoke('send_command', { ismaster: false, command: commandslave+'\n' })
    }

    async function signalSlave() {
        await invoke('send_signal', { ismaster: false, signal: "INT" })
    }
    async function scanDevice(ismaster) {
        // Переход на страницу редактирования устройства
        await invoke('scan_device', { ismaster });
        if (ismaster) {
            open_master_ip = true
            masterStep = 1
        } else {
            open_svave_ip = true
        }
        all_ip_master = await get_device_ip(true);
        all_id_slave = await get_device_ip(false);
        progress += 1
        progressMain +=1
    }
    async function ArchiveGen(ismaster) {
        // Переход на страницу редактирования устройства
        await invoke('create_archive', { ismaster });
        masterStep = 2
        progress += 1
        progressMain +=1
    }

    async function restartSession (ismaster) {
        
        await invoke('restart_session', { ismaster });
    }

    async function downloadFile(ismaster) {
        
        if (pahtdownload === "") { 
            pahtdownload = await open({
                multiple: false,
                directory: true,
            });
        }
        const  fileName  = await invoke('download_file', {filepath:pahtdownload, ismaster:ismaster });
        if (fileName) {
            // Показать всплывающее окно с адресом папки загрузок
            showNotification(`архив заргужен в папку загрузок по пути: ${fileName}`);
            
        } else {
            showNotification('Не удалось скачать файл.');
        }
        progress += 1
        progressMain +=1
        
    }

    console.log(all_ip_master);

    async function ste_default() {
        all_ip_master = await get_device_ip(true);
        all_id_slave = await get_device_ip(false);
    }



    async function changeScroll(ismaster) {
        if (ismaster) {
            autoScrollMaster = !autoScrollMaster;
        } else {
            autoScrollSlave = !autoScrollSlave;
        }
    }

    function scrollToBottomMaster() {
        if (terminalContainerMaster && autoScrollMaster) {
            terminalContainerMaster.scrollTop = terminalContainerMaster.scrollHeight;
        }
    }
    function scrollToBottomSlave() {
        if (terminalContainerSlave && autoScrollSlave) {
            terminalContainerSlave.scrollTop = terminalContainerSlave.scrollHeight;
        }
    }

    

    // Автопрокрутка каждый раз при изменении TerminalMaster
    $: TerminalMaster, scrollToBottomMaster();
    $: TerminalSlave, scrollToBottomSlave();



    onMount(
        ste_default
    )
</script>



<style>
    .scrollable-container {
        overflow-y: auto;
        height: 100%;
    }

    .custom_bg_master {
        background-color: #91e1fa;
    }
    .custom_bg_slave {
        background-color: #d4d4d4;
    }
    .custom-card-body{
        padding: 7px;
        height: 110px;
    }
    .terminal_button {
        position: absolute;
        bottom: 10px; 
        right: 10px;
        height: 40px;
        width: 135px;
    }
    .terminal {
        border-radius: 5px;
    }
    .text{
        width: 480px; 
    }
    .alarm-text{
        color: #f80000;
    }

    .custom-line{
        width: 100%;
        background-color: rgba(180, 191, 255, 0.12); /* Полупрозрачный фон */
        padding: 20px;
        position: relative;
        
    }

    .vertical-text {
        writing-mode: vertical-rl; /* Вертикальная ориентация текста */
        transform: rotate(180deg); /* Поворачиваем текст */
        position: absolute;
        color: rgb(71, 69, 69);
        font-size: 18px;
        top: 50%;
        
        transform: translateY(-50%) rotate(180deg); /* Центрируем по вертикали и поворачиваем текст */
    }
    .left {
        left: 10px;
    }
    .right {
        right: 10px;
    }

    .notification {
        position: fixed;
        bottom: 20px;
        right: 20px;
        background-color: #51acac; /* Зеленый фон */
        color: white;
        padding: 15px;
        border-radius: 5px;
        box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
        z-index: 1000; /* Чтобы уведомление было поверх других элементов */
        transition: opacity 0.5s ease-in-out;
    }

    .notification.hidden {
        opacity: 0;
        visibility: hidden;
    }

    .notification.visible {
        opacity: 1;
        visibility: visible;
    }

.custom-master_terminal{
    background-color: #f80000;
}
</style>



<div class="custom-line">
    <div class="vertical-text left">3_Установка</div>
    <div class="container" style="width: 100%">
        
        <div class="row" >
            <div class="{maxSizeMaster ? 'col-md-12' : 'col-md-6'}">
                <div class="card bg-info text-dark bs-gradient" style="width: 100%; {maxSizeMaster ? 'height: 700px' : 'height: 450px'}">
                    <div id="notification" class="notification hidden">
                        <span id="notification-text"></span>
                    </div>
                    <div class="d-flex justify-content-between align-items-center">
                        <h6>Терминал Мастер</h6> 
                        пропустить до:
                        <div class="col-md-3">
                            <select class="form-select" bind:value={installation_stage_master}>
                                {#each  installation_stage as  st, index}
                                <option value={index}>{st}</option>
                                {/each}
                            </select>
                        </div>
                        <button class="btn btn-secondary btn-sm" type="button" on:click={() => change_state(true)} >|></button>
                        <div class="btn-group me-2" role="group" aria-label="First group">
                            <button class="btn btn-sm {autoScrollMaster? 'btn-outline-primary':'btn-outline-warning'}" type="button" on:click={() => changeScroll(true)} >>|</button>
                            <!--{#if stateDevises.master_code != "Expectation Ok"}-->
                            {#if /^Expectation \d+$/.test(stateDevises.master_code)}
                                <button class="btn btn-secondary btn-sm" type="button" on:click={() => restartSession(true)} disabled>-></button>
                            {:else}
                                <button class="btn btn-secondary btn-sm" type="button" on:click={() => restartSession(true)} >-></button>
                            {/if}
                            {#if maxSizeMaster }
                                <button class="btn btn-primary btn-sm" type="button" on:click={() => maxSizeMaster=false}>-</button>
                            {:else}
                                <button class="btn btn-primary btn-sm" type="button" on:click={() => maxSizeMaster=true}>+</button>
                            {/if}
                        </div>
                    </div>
                
                    <div class="form-group mb-4">
                        <div class="input-group">
                            <input id="greet-input" type="text" class="form-control" placeholder="Введите команду для отправки на сервер" bind:value="{commandmaster}" />
                            <!--{#if stateDevises.master_code === "Expectation OK"}-->
                            {#if /^Expectation \d+$/.test(stateDevises.master_code)}
                                <button on:click="{sendMaster}"  disabled >Выполнить</button>
                                <button on:click="{signalMaster}" disabled >Ctrl+C</button>
                            {:else}
                                <button on:click="{sendMaster}" >Выполнить</button>
                                <button on:click="{signalMaster}">Ctrl+C</button>
                            {/if}
                        </div>
                    </div>
                    
                    <div class="scrollable-container " bind:this={terminalContainerMaster}>
                        {#each TerminalMaster as message}
                            {message.message}
                            <br>
                        {/each}
                    </div>

                    <div class="d-grid gap-2 d-md-flex justify-content-md-end ">
                        <div class="card " style="width: 100%; height: 110px; position: relative;">
                            <div class="custom-card-body terminal custom_bg_master">
                                {#if stateDevises.master_state === "CREATE"}
                                    
                                    {#if stateDevises.master_code === "Success 0"}
                                    <h5>Шаг: 1</h5>
                                        <p>Отсканируйте сервер</p>
                                        <button class="btn btn-primary terminal_button" type="button" on:click={() => scanDevice(true)}>Cканировать</button>
                                    {:else}
                                    <h5>Шаг: 0</h5>
                                    <p class="text">Зайдите на сервер в разделе '2_Сервера'</p>
                                    {/if}
                                {:else if stateDevises.master_state === "SCAN"}
                                    
                                    {#if stateDevises.master_code ==="Success 0"}
                                    <h5>Шаг: 2.1</h5>
                                        {#if ActCompanyName === "Not Found"}
                                        <p class="text alarm-text">Для создания архива используется название компании. Укажите название вашей компании разделе '1_Компания'</p>
                                        
                                                
                                                <button class="btn btn-primary terminal_button" disabled type="button" 
                                                
                                                on:click={() => ArchiveGen(true)} >Создать архив</button>
                                            
                                        {:else}
                                        <p class="text">Создайте архив для получения лицензии</p>
                                                <button class="btn btn-primary terminal_button" type="button"
                                                
                                                on:click={() => ArchiveGen(true)} >Создать архив</button>
                                            
                                        
                                        {/if}
                                    {:else}
                                        <h5>Шаг: 1 (сканирование)</h5>
                                        <Progress progress={stateDevises.master_code} maxStage={6} />
                                    {/if}

                                {:else if stateDevises.master_state === "ARCHIVEGENERATION"}
                                    {#if stateDevises.master_code ==="Success 0"}
                                        <h5>Шаг: 2.2</h5>
                                        <p class="text">Архив был успешно создан. Скачайте его для обмена на файлы лицензии</p>
                                            <button class="btn btn-primary terminal_button" type="button"
                                            on:click={() => downloadFile(true)} >Скачать архив</button>
                                    {:else}
                                        <h5>Шаг: 2.1 (Создаю архив)</h5>
                                        <Progress progress={stateDevises.master_code} maxStage={1} />
                                    {/if}
                                
                                {:else if stateDevises.master_state === "ARCHIVEDOWNLOAD"}
                                    {#if stateDevises.master_code ==="Success 0"}
                                        {#if fileMasterLic === ""}
                                        <h5>Шаг: 3.1</h5>
                                            <p class="text">Укажите файл лицензии (.lic)</p>
                                            
                                            <button class="btn btn-warning terminal_button" on:click={() => SelectSwkLic(true)}>Выбрать</button>
                                        
                                        {:else if fileMasterSwk === ""}
                                        <h5>Шаг: 3.2</h5>
                                        <p class="text">Укажите файл лицензии (.swk)</p>
                                            <button class="btn btn-success terminal_button" on:click={() => SelectSwkLic(true)}>Выбрать</button>
                                        {:else}
                                        <h5>Шаг: 3.3</h5>
                                        <p class="text">Отправьте файлы лицензии на сервер</p>
                                            <button class="btn btn-primary terminal_button" on:click={() => uploadFile(true)}>Отправить</button>
                                        {/if}
                                    {:else}
                                    {/if}
                                {:else if stateDevises.master_state === "SENDINGFILES"}
                                    {#if stateDevises.master_code ==="Success 0"}
                                        <h5>Шаг: 4</h5>
                                        {#if masterIp !== "" && slaveIp !=="" && rsPort !== null}
                                        <p class="text">Установите ПО</p>
                                        <button class="btn btn-primary terminal_button" on:click={() => start_installate(true)}>Установить</button>
                                        {:else}
                                        <p class="text alarm-text">Сервер готов к установке ПО. Укажите IP Мастера, IP Слейва и Rsync-порт в разделе "4_Конфигурация_РТУ"</p>
                                        {/if}
                                    {:else}
                                    {/if}
                                {:else if stateDevises.master_state === "INSTALLATE"}
                                    {#if stateDevises.master_code ==="Success 0"}
                                        <h5>Шаг: 5</h5>
                                            
                                            {#if is_install_master}
                                                {#if swimming_ip !== "" && subnetMaskIdex}
                                                    <p class="text">Требуется настройка ПО</p>
                                                    <button class="btn btn-primary terminal_button" on:click={() => start_config(true)}>Запустить</button>
                                                {:else}
                                                    <p class="text alarm-text">Укажите "Плавающий IP" в разделе "4_Конфигурация_РТУ"</p>
                                                {/if}
                                            {/if}
                                    
                                    {:else}
                                        <h5>Шаг: 4 (Установка)</h5>
                                        <Progress progress={stateDevises.master_code} maxStage={13626} />
                                    {/if}
                                {:else if stateDevises.master_state === "CONFIGURATOR"}
                                    {#if stateDevises.master_code ==="Success 0"}
                                    <h5>Шаг: 5</h5>
                                    <h6>ПО успешно установлено на Мастер. Если ПО уже установлено на Слейв переходите к разделу "5_Синхронизация"</h6>
                                    {/if}
                                {:else}
                                    <h5>Шаг: 5</h5>
                                    <h6>ПО успешно установлено на Мастер. Если ПО уже установлено на Слейв переходите к разделу "5_Синхронизация"</h6>
                                {/if}    
                            </div>
                        </div>
                    </div>            
                </div>
            </div>

            <div class="{maxSizeSlave ? 'col-md-12' : 'col-md-6'}">
                <div class="card bg-secondary text-white {maxSizeSlave ? 'col-md-12' : 'col-md-6'}" style="width: 100%; {maxSizeSlave ? 'height: 700px' : 'height: 450px'}">
                    <div class="d-flex justify-content-between align-items-center">
                        <h6>Терминал Слейв</h6> 
                        пропустить до:
                        <div class="col-md-3">
                            <select class="form-select" bind:value={installation_stage_slave}>
                                {#each  installation_stage as  st, index}
                                <option value={index}>{st}</option>
                                {/each}
                            </select>        
                        </div>
                        
                        <button class="btn btn-primary btn-sm" type="button" on:click={() => change_state(false)}>|></button>
                        <div class="btn-group me-2" role="group" aria-label="First group">
                            <button class="btn btn-sm {autoScrollSlave? 'btn-outline-primary':'btn-outline-warning'}" type="button" on:click={() => changeScroll(false)} >>|</button>
                            {#if /^Expectation \d+$/.test(stateDevises.slave_code)}
                                <button class="btn btn-secondary btn-sm" type="button" on:click={() => restartSession(false)} disabled>-></button>
                            
                            {:else}
                                <button class="btn btn-secondary btn-sm" type="button" on:click={() => restartSession(false)} >-></button>
                            {/if}
                            {#if maxSizeSlave }
                                <button class="btn btn-primary btn-sm" type="button" on:click={() => maxSizeSlave=false}>-</button>
                            {:else}
                                <button class="btn btn-primary btn-sm" type="button" on:click={() => maxSizeSlave=true}>+</button>
                            {/if}
                        </div>
                    </div>
                    
                    <div class="form-group mb-4">
                        <div class="input-group">

                            <input id="greet-input" type="text" class="form-control" placeholder="Введите команду для отправки на сервер" bind:value="{commandslave}" />
                            {#if /^Expectation \d+$/.test(stateDevises.slave_code)}

                                <button on:click="{sendSlave}" disabled>Выполнить</button>
                                <button on:click="{signalSlave}" disabled>Ctrl+C</button>
                            {:else}
                                <button on:click="{sendSlave}">Выполнить</button>
                                <button on:click="{signalSlave}">Ctrl+C</button>
                            {/if}
                        </div>
                    </div>
                    <div class="scrollable-container " bind:this={terminalContainerSlave}>
                        {#each TerminalSlave as message}
                            {message.message}
                            <br>
                        {/each}
                    </div>
                    <div class="d-grid gap-2 d-md-flex justify-content-md-end ">
                        <div class="card " style="width: 100%; height: 110px; position: relative;">
                            <div class="custom-card-body terminal custom_bg_slave">
                                {#if stateDevises.slave_state === "CREATE"}
                                    {#if stateDevises.slave_code ==="Success 0"}
                                        <h5>Шаг: 1</h5>
                                        <p>Отсканируйте сервер</p>
                                        <button class="btn btn-primary terminal_button" type="button" on:click={() => scanDevice(false)} >Cканировать</button>
                                    {:else}
                                    <h5>Шаг: 0</h5>
                                    <p class="text">Зайдите на сервер в разделе '2_Сервера'</p>
                                    {/if}
                            
                                {:else if stateDevises.slave_state === "SCAN"}
                                    {#if stateDevises.slave_code ==="Success 0"}
                                        <h5>Шаг: 2.1</h5>
                                        {#if ActCompanyName === "Not Found"}
                                            <p class="text alarm-text">Для создания архива используется название компании. Укажите название вашей компании в разделе '1_Компания'</p>
                                            <button class="btn btn-primary terminal_button" disabled type="button" on:click={() => ArchiveGen(false)} >Создать архив</button>
                                            
                                        {:else}
                                            <p class="text">Создайте архив для получения лицензии</p>
                                            <button class="btn btn-primary terminal_button" type="button" on:click={() => ArchiveGen(false)} >Создать архив</button>
                                        {/if}
                                    {:else}
                                        <h5>Шаг: 1 (сканирование)</h5>
                                        <Progress progress={stateDevises.slave_code} maxStage={6} />
                                    {/if}

                                {:else if stateDevises.slave_state === "ARCHIVEGENERATION"}
                                
                                    {#if stateDevises.slave_code ==="Success 0"}
                                        <h5>Шаг: 2.2</h5>
                                        <p class="text">Архив был успешно создан. Скачайте его для обмена на файлы лицензии</p>
                                        <button class="btn btn-primary terminal_button" type="button" on:click={() => downloadFile(false)} >Скачать архив</button>
                                    {:else}
                                        <h5>Шаг: 2.1 (Создаю архив)</h5>
                                        <Progress progress={stateDevises.slave_code} maxStage={1} />
                                    {/if}
                            
                                {:else if stateDevises.slave_state === "ARCHIVEDOWNLOAD"}
                                    {#if stateDevises.slave_code ==="Success 0"}
                                        {#if fileSlaveLic === ""}
                                            <h5>Шаг: 3.1</h5>
                                            <p class="text">Укажите файл лицензии (.lic)</p>
                                            <button class="btn btn-warning terminal_button" on:click={() => SelectSwkLic(false)}>Выбрать</button>
                                
                                        {:else if fileSlaveSwk === ""}
                                            <h5>Шаг: 3.2</h5>
                                            <p class="text">Укажите файл лицензии (.swk)</p>
                                            <button class="btn btn-success terminal_button" on:click={() => SelectSwkLic(false)}>Выбрать</button>
                                        {:else}
                                            <h5>Шаг: 3.3</h5>
                                            <p class="text">Отправьте файлы лицензии на сервер</p>
                                            <button class="btn btn-primary terminal_button" on:click={() => uploadFile(false)}>Отправить</button>
                                        {/if}
                                    {:else}
                                    {/if}
                                
                                {:else if stateDevises.slave_state === "SENDINGFILES"}
                                    {#if stateDevises.slave_code ==="Success 0"}
                                        <h5>Шаг: 4</h5>
                                        {#if masterIp !== "" && slaveIp !=="" && rsPort !== null}
                                            <p class="text">Установите ПО</p>
                                            <button class="btn btn-primary terminal_button" on:click={() => start_installate(false)}>Установить</button>
                                        {:else}
                                            <p class="text alarm-text">Сервер готов к установке ПО. Укажите IP Мастера, IP Слейва и Rsync-порт в разделе "4_Конфигурация_РТУ"</p>
                                        {/if}
                                    {:else}
                                    {/if}
                                {:else if stateDevises.slave_state === "INSTALLATE"}
                                    {#if stateDevises.slave_code ==="Success 0"}
                                        <h5>Шаг: 5</h5>
                                        {#if swimming_ip !== "" && subnetMaskIdex}
                                            <p class="text">Требуется настройка ПО</p>
                                            <button class="btn btn-primary terminal_button" on:click={() => start_config(false)}>Запустить</button>
                                        {:else}
                                            <p class="text alarm-text">Укажите "Плавающий IP" в разделе "4_Конфигурация_РТУ"</p>
                                        {/if}
                                        
                                        
                                    {:else}
                                    <h5>Шаг: 4 (Установка)</h5>
                                    <Progress progress={stateDevises.slave_code} maxStage={13626} />
                                    {/if}
                                {:else if stateDevises.slave_state === "CONFIGURATOR"}
                                    {#if stateDevises.slave_code ==="Success 0"}
                                    <h5>Шаг: 5</h5>
                                        <h6>ПО успешно установлено на Слейв. Если ПО уже установлено на Мастер переходите к разделу "5_Синхронизация"</h6>
                                    {/if}
                                {:else}
                                <h5>Шаг: 5</h5>
                                <h6>ПО успешно установлено на Слейв. Если ПО уже установлено на Мастер переходите к разделу "5_Синхронизация"</h6>
                                {/if}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            

        </div>
        
    </div>
    
</div>


<br>


<div class="custom-line">
    <div class="vertical-text left">4_Конфигурация_Нод</div>
        <div class="container">
            <div class="card bg-secondary text-white">
                <div class="card-body ">
                    <div class="row">
                        <div class="col-md-4">
                            <label>IP Мастера: </label>
                            {#if open_master_ip}
                            <select class="form-select" name="ipMaster" aria-label="Default select example" bind:value={masterIp}>
                                {#each all_ip_master as ip}
                                <option value={ip}>{ip}</option>
                                {/each}
                            </select>
                            {:else}
                            <select class="form-select" name="ipMaster" aria-label="Default select example" disabled>
                                <option selected>{masterIp}</option>
                            </select>
                            {/if}
                        </div>
                        <div class="col-md-4">
                            <label>IP Слейва: </label>
                            {#if open_svave_ip}
                            <select  class="form-select" name="slaveIp" aria-label="Default select example" bind:value={slaveIp}>
                                {#each all_id_slave as ip}
                                <option value={ip}>{ip}</option>
                                {/each}
                            </select>
                            {:else}
                            <select  class="form-select" name="slaveIp" aria-label="Default select example" disabled>
                                <option selected>{slaveIp}</option>
                            </select>
                            {/if}
                        </div>
                        {#if !is_protectedRsPort}
                        <div class="col-md-4">
                            <label >Порт</label>
                            
                            <input type="number" class="form-control" id="inputZip" value={rsPort} >
                        </div>
                        {:else}
                        <div class="col-md-4">
                            <label >Rsync Порт</label>
                            
                            <input type="number" class="form-control" id="inputZip" value={rsPort} disabled>
                        </div>
                        {/if}
                        <div class="col-md-4">
                            <label >Плавающий IP {swimming_ip}</label>
                            {#if open_swimming_ip }
                                <input type="text" class="form-control"  bind:value={swimming_ip}>
                            {:else}
                                <input type="text" class="form-control"  bind:value={swimming_ip} disabled>
                            {/if}
                        </div>
                        <div class="col-md-4">
                            <label >Маска плавающего IP {swimming_ip}</label>
                            
                            <select class="form-select" bind:value={subnetMaskIdex} disabled={!open_swimming_ip}>
                                {#each  subnetMasks as  st, index}
                                <option value={index}>{st}</option>
                                {/each}
                            </select>
                        </div>
                        <div class="col-md-4">
                            <label >Шлюз доступности сети IP {pingGw}</label>
                            {#if open_pingGw }
                            <input type="text" class="form-control" bind:value={pingGw}>
                            {:else}
                                <input type="text" class="form-control"  bind:value={pingGw} disabled>
                            {/if}
                        </div>
                    </div>
                </div>
            </div>   
        </div>
        
    </div>
<br>
<div class="custom-line">
    <div class="vertical-text left">5_Объединение_кластера</div>
    <div class="container"  style="width: 100%; height: 205px;">
        <div class="card text-center" style="width: 100%; height: 200px; position: relative;">
                {stateDevises.slave_state} {stateDevises.master_state} {stateDevises.master_code} {stateDevises.slave_code}
                {#if stateDevises.slave_state === "CONFIGURATOR" && stateDevises.master_state === "CONFIGURATOR"}
                    {#if stateDevises.master_code ==="Success 0" && stateDevises.slave_code ==="Success 0"}
                    <div class="card-header">
                        <h5>Шаг 6 (синхронизация БД)</h5>
                    </div>
                    <div class="card-body">
                        <p>Для настройки кластера необходимо синхронизировать БД</p>
                        <div class="d-grid gap-2 col-6 mx-auto">
                            <button class="btn btn-warning me-md-2" type="button" on:click={() => start_config_db()}>Синхронизировать БД</button>
                        </div>
                    </div>
                    {:else}
                        <div class="card-header">
                            <h5>Шаг 5</h5>
                        </div>
                        <div class="card-body">
                            <br>
                                <ProgressBig progressMaster = {stateDevises.master_code}, progressSlave ={stateDevises.slave_code}, maxStage=20/>
                            
                        </div>
                    {/if}
                        
                        
                {:else if stateDevises.slave_state === "DBCONFIGURATOR" && stateDevises.master_state === "DBCONFIGURATOR"}
                    {#if stateDevises.master_code ==="Success 0" && stateDevises.slave_code ==="Success 0"}
                        <div class="card-header">
                            <h5>Шаг 7 (синхронизация кластера)</h5>
                        </div>
                        {#if pingGw && masterIp && slaveIp && swimming_ip && subnetMaskIdex}
                        <div class="card-body">
                            <div class="d-grid gap-2 col-6 mx-auto">
                                <button class="btn btn-warning me-md-2" type="button" on:click={() => start_coropace()}>Синхронизировать PTУ</button>
                            </div>                            
                        </div>
                        {:else}
                        <p class="alarm-text">Убедитесь, что в разделе "4_Конфигурация_РТУ" заполнены следующие поля: <br> "IP Мастера", "IP Слейва", "Плавающий IP", "Маска плавающего IP" "Шлюз доступности сети IP".</p>
                        {/if}
                    {:else}
                        <div class="card-header">
                            <h5>Шаг 6 (синхронизация БД)</h5>
                        </div>
                        <div class="card-body">
                            <br>
                                <ProgressBig progressMaster = {stateDevises.master_code}, progressSlave ={stateDevises.slave_code}, maxStage=44/>
                            
                        </div>
                    {/if}
                {:else if stateDevises.slave_state === "COROPASE" && stateDevises.master_state === "COROPASE"}
                    {#if stateDevises.master_code ==="Success 0" && stateDevises.slave_code ==="Success 0"}
                        <div class="card-header">
                            <h5>Шаг 8 </h5>
                        </div>
                        <div class="card-body">
                            <p>Установка завершена</p>
                        </div>
                    {:else}
                        <div class="card-header">
                            <h5>Шаг 7 (синхронизация кластера)</h5>
                        </div>
                        <div class="card-body">
                            <br>
                                <ProgressBig progressMaster = {stateDevises.master_code}, progressSlave ={stateDevises.slave_code}, maxStage=15/>
                            
                        </div>
                    {/if}

                {:else}
                
                    <div class="card-header">
                        <h5>Отсутствует ПО</h5>
                    </div>
                    <div class="card-body">
                        <p>Отсутствует или не полностью настроено ПО на Мастере или Слейве</p>
                    </div>

                    
                {/if}
                <br>
                    
            </div>  
            
                
            
        </div>
        
    </div>
    <br>
    <div class="position-fixed bottom-0 w-100  text-white text-center py-3">
    <ProgressMain progress =  {progressMain} maxStage=60/>
</div>



