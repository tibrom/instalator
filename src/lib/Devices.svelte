<script lang="ts">
    import { invoke } from '@tauri-apps/api';
    import { onMount, } from 'svelte';
  
    
  
    interface Device {
      id: number;
      ip_address: string | null;
      credentials: { username: string; password: string; sudopassword: string} | null;
      state: { Creating?: string; Scanning?: string; Settings?: string; Gone? } | null;
      port: number;
      device_type: string | null;
    }
  
    let masterdevices: Device |null = null;
    let slavedevice: Device |null = null;
    

    


  
    async function fetchDevices() {
      try {
        masterdevices = await invoke<Device>('get_device', {ismaster: true});
        slavedevice = await invoke<Device>('get_device', {ismaster: false});
      } catch (error) {
        console.error('Failed to fetch devices:', error);
      }
    }

    let masterip_address = "";
    let master_port = 22;
    let master_username = "admin";
    let master_password = null;
    let master_sudopassword = null;
    let master_file_key: FileList | null = null;
    let master_file_content = "";

    let slave_address = "";
    let slave_port = 22;
    let slave_username = "admin";
    let slave_password = null;
    let slave_sudopassword = null;
    let slave_file_key: FileList | null = null;
    let slave_file_content = "";
  

    
    

    // Функция для чтения содержимого файла и преобразования его в строку
    function readFileAsText(file: File): Promise<string> {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = () => resolve(reader.result as string);
            reader.onerror = reject;
            reader.readAsText(file);
        });
    }

    async function createDevice(ismaster) {
        if (ismaster) {
            if (master_file_key && master_file_key.length > 0) {
                // Преобразуем первый файл в строку
                master_file_content = await readFileAsText(master_file_key[0]);
            }
            await invoke('create_device', {
                ipaddress: masterip_address,
                port: master_port,
                username: master_username,
                password: master_password,
                sudopassword: master_sudopassword,
                fileKey: master_file_content,
                ismaster: true
            });
        } else {
            if (slave_file_key && slave_file_key.length > 0) {
                // Преобразуем первый файл в строку
                slave_file_content = await readFileAsText(master_file_key[0]);
            }
            await invoke('create_device', {
                ipaddress: slave_address,
                port: slave_port,
                username: slave_username,
                password: slave_password,
                sudopassword: slave_sudopassword,
                fileKey: slave_file_content,
                ismaster: false
            });
        }
        fetchDevices();
    }

    async function delete_device(ismaster) {
        await invoke('delete_device', { ismaster })
        fetchDevices();
    }

    
  
    onMount(fetchDevices);
  </script>
  
  <div class="container">
    
    <div class="row">
        <div class="col-md-6 ">
            <h5 class="card-title text-center mt-3 " >Мастер</h5>
            <div class="card bg-info text-dark bs-gradient" >
                
                {#if masterdevices }
                <div class="card" style="width: 100%; height: 280px">
            
                <table class="table table-info table-sm ">
                    <tbody>
                        <tr>
                            <td>Ip адрес</td>
                            <td><div class="">
                                <input type="text" name="host" class="form-control" value={masterdevices.ip_address} disabled>
                            </div>
                            </td>
                        </tr>
                        <tr>
                            <td>Порт</td>
                            <td><div class="">
                                <input type="text" name="host" class="form-control" value={masterdevices.port} disabled>
                            </div>
                            </td>
                        </tr>
                        <tr>
                            <td>Пользователь</td>
                            <td><div class="">
                                <input type="text"   class="form-control" value={masterdevices.credentials?.username ?? 'N/A'} disabled>
                            </div>
                            </td>
                        </tr>
                        <tr>
                            <td>Пароль</td>
                            <td><div class="">
                                <input type="password"   class="form-control" value={masterdevices.credentials?.password ?? 'N/A'} disabled>
                            </div>
                            </td>
                        </tr>
                        <tr>
                            <td>Sudo Пароль</td>
                            <td><div class="">
                                <input type="password"   class="form-control" value={masterdevices.credentials?.sudopassword ?? 'N/A'} disabled>
                            </div>
                            </td>
                        </tr>
                        <tr>
                            <td>ssh ключ файл</td>
                            <td><div class="">
                                <input type="text"   class="form-control"  disabled>
                            </div>
                            </td>
                        </tr>
                        
                    </tbody>
                </table>
                </div>
                <div class="d-grid gap-2 d-md-flex justify-content-md-end ">
                    <div class="btn-group  me-md-2 mb-3 mt-3 me-3 ms-3">
                        <a class="btn btn-primary" type="button" on:click={() => delete_device(true)}>Выйти</a>
                    </div>
                </div>
                {:else}
                
                    <div class="card" style="width: 100%; height: 280px">
                        <table class="table table-info table-sm">
                            <tbody>
                                <tr>
                                    <td>Ip адрес</td>
                                    <td><div class="">
                                        <input type="text" placeholder="192.168.0.0" name="host" class="form-control" bind:value={masterip_address}>
                                    </div>
                                    </td>
                                </tr>
                                <tr>
                                    <td>Порт</td>
                                    <td><div class="input-group">
                                        <input type="number" name="port" class="form-control"  step="1" max="65535" min="1" bind:value={master_port}>
                                    </div>
                                    </td>
                                </tr>
                                <tr>
                                    <td>Пользователь</td>
                                    <td><div class="input-group">
                                        <input type="text" placeholder="admin" name="username" class="form-control" bind:value={master_username}>
                                    </div>
                                    </td>
                                </tr>
                                <tr>
                                    <td>Пароль</td>
                                    <td><div class="input-group">
                                        <input type="password" name="password" class="form-control" bind:value={master_password}>
                                    </div>
                                    </td>
                                </tr>
                                <tr>
                                    <td>Sudo Пароль</td>
                                    <td><div class="input-group">
                                        <input type="password"  name="sudopassword" class="form-control" bind:value={master_sudopassword}>
                                    </div>
                                    </td>
                                </tr>
                                <tr>
                                    <td>ssh ключ, файл</td>
                                    <td><input type="file" class="form-control" name="file_key" bind:value={master_file_key}>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                        
                    </div>

                    <div class="d-grid gap-2 d-md-flex justify-content-md-end ">
                        <div class="btn-group  me-md-2 mb-3 mt-3 me-3 ms-3">
                            <a class="btn btn-primary" type="button" on:click={() => createDevice(true)}>Войти</a>
                        </div>
                    </div>
                {/if}
            </div>
        </div>
        <div class="col-md-6">
            <h5 class="card-title text-center mt-3">Слейв</h5>
            <div class="card bg-secondary text-whit" style="width: 100%">
                {#if slavedevice }
                <div class="card" style="width: 100%; height: 280px">
            
                    <table class="table table-secondary table-sm ">
                        <tbody>
                            <tr>
                                <td>Ip адрес</td>
                                <td><div class="">
                                    <input type="text" name="host" class="form-control" value={slavedevice.ip_address} disabled>
                                </div>
                                </td>
                            </tr>
                            <tr>
                                <td>Порт</td>
                                <td><div class="">
                                    <input type="text" name="host" class="form-control" value={slavedevice.port} disabled>
                                </div>
                                </td>
                            </tr>
                            <tr>
                                <td>Пользователь</td>
                                <td><div class="">
                                    <input type="text"   class="form-control" value={slavedevice.credentials?.username ?? 'N/A'} disabled>
                                </div>
                                </td>
                            </tr>
                            <tr>
                                <td>Пароль</td>
                                <td><div class="">
                                    <input type="password"   class="form-control" value={slavedevice.credentials?.password ?? 'N/A'} disabled>
                                </div>
                                </td>
                            </tr>
                            <tr>
                                <td>Sudo Пароль</td>
                                <td><div class="">
                                    <input type="password"   class="form-control" value={slavedevice.credentials?.sudopassword ?? 'N/A'} disabled>
                                </div>
                                </td>
                            </tr>
                            <tr>
                                <td>ssh ключ файл</td>
                                <td><div class="">
                                    <input type="text"   class="form-control"  disabled>
                                </div>
                                </td>
                            </tr>
                            
                        </tbody>
                    </table>
                    </div>
                    <div class="d-grid gap-2 d-md-flex justify-content-md-end ">
                        <div class="btn-group  me-md-2 mb-3 mt-3 me-3 ms-3">
                            <a class="btn btn-primary" type="button" on:click={() => delete_device(false)}>Выйти</a>
                        </div>
                    </div>
                {:else}
                <div class="card" style="width: 100%; height: 280px">
                    <table class="table table-secondary table-sm ">
                        <tbody>
                            <tr>
                                <td>Ip адрес</td>
                                <td><div class="">
                                    <input type="text" placeholder="192.168.0.0" name="host" class="form-control" bind:value={slave_address}>
                                </div>
                                </td>
                            </tr>
                            <tr>
                                <td>Порт</td>
                                <td><div class="input-group">
                                    <input type="number" name="port" class="form-control"  step="1" max="65535" min="1" bind:value={slave_port}>
                                </div>
                                </td>
                            </tr>
                            <tr>
                                <td>Пользователь</td>
                                <td><div class="input-group">
                                    <input type="text" placeholder="admin" name="username" class="form-control" bind:value={slave_username}>
                                </div>
                                </td>
                            </tr>
                            <tr>
                                <td>Пароль</td>
                                <td><div class="input-group">
                                    <input type="password" name="password" class="form-control" bind:value={slave_password}>
                                </div>
                                </td>
                            </tr>
                            <tr>
                                <td>Sudo Пароль</td>
                                <td><div class="input-group">
                                    <input type="password"  name="sudopassword" class="form-control" bind:value={slave_sudopassword}>
                                </div>
                                </td>
                            </tr>
                            <tr>
                                <td>ssh ключ, файл</td>
                                <td><input type="file" class="form-control" name="file_key" bind:value={slave_file_key}>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                    
                </div>

                <div class="d-grid gap-2 d-md-flex justify-content-md-end ">
                    <div class="btn-group  me-md-2 mb-3 mt-3 me-3 ms-3">
                        <a class="btn btn-primary" type="button" on:click={() => createDevice(false)}>Войти</a>
                    </div>
                </div>
                {/if}
            </div>
        </div>
    </div>
</div>








