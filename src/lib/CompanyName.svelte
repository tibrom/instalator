<script>
    import { invoke } from '@tauri-apps/api/tauri'
    import { onMount, } from 'svelte';
    export let stateDevises;
  
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