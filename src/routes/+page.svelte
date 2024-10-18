<script lang="ts"> 
    import Greet from '../lib/Greet.svelte';
    import Devices from '../lib/Devices.svelte';
    import Alert from '../lib/Alert.svelte';
    import Terminals from '../lib/Terminals.svelte';
    import CompanyName from '../lib/CompanyName.svelte';
    import { invoke } from '@tauri-apps/api';
    import { onMount } from 'svelte';
    
    // StateData Struct аналог
    interface StateData {
        master_state: string;
        slave_state: string;
        master_code: string; // Исправлено на string
        slave_code: string; // Исправлено на string
    }

    let stateDevises: StateData = {
        master_state: '',
        slave_state: '',
        master_code: '',
        slave_code: ''
    };
    
    
    // Определение типа AlertMessage, если его еще нет
    interface AlertMessage {
        belong: string;
        message: string;
    }

    let TerminalSlave: AlertMessage[] = [];
    let TerminalMaster: AlertMessage[] = [];
    

    async function fetchMessages() {
        TerminalSlave = await invoke<AlertMessage[]>('get_terminal_slave');
        TerminalMaster = await invoke<AlertMessage[]>('get_terminal_master');
        stateDevises = await invoke<StateData>('get_state');
    }

    // Запрашиваем сообщения через регулярные интервалы
    setInterval(fetchMessages, 300);
    // onMount(fetchMessages); // для начального запроса
    let companyname = ''
    let ActCompanyName = ''

    async function getName() {
        ActCompanyName = await invoke('get_company_name')
    }
  
    async function set_name() {
        ActCompanyName = await invoke('create_company_name', {companyname})
    }
    onMount(getName);
</script>

<style>
.custom-line{
    display: flex;
    justify-content: center;  /* Центрируем по горизонтали */
    align-items: center;      /* Центрируем по вертикали */
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

</style>



<!--<Devices />-->
<br>
<div class="custom-line">
    <div class="vertical-text left">1_Компания</div>
        <div class="container">
            <p>название компании</p>
            <h3>{ActCompanyName}</h3>
            {#if (stateDevises.slave_state === "CREATE" || stateDevises.slave_state === "SCAN") && 
                (stateDevises.master_state === "CREATE" || stateDevises.master_state === "SCAN")}
                
            <div class="form-group mb-4">
                <label for="companyname" >Изменить название компании</label>
                <div class="input-group">
                    <input type="text" name="companyname" placeholder="ООО 'Рога и Копыта'" class="form-control" bind:value="{companyname}">
                    <button class="btn btn-primary" on:click="{set_name}">Применить</button>
                </div>
            </div>
            {/if}
            <p>Название компании применяется для генерации лицензии</p>
        </div> 
    

</div>

<br>
<div class="custom-line">
    <div class="vertical-text left">2_Сервера</div>
        <Devices />
   
</div>

<br>
<Terminals {TerminalSlave} {TerminalMaster} {stateDevises} {ActCompanyName}/>
<br>
