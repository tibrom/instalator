<script lang="ts">
    import { invoke } from '@tauri-apps/api';
    import { page } from '$app/stores';
    import { derived } from 'svelte/store';
    

    let ip_address = "";
    let port = 22;
    let username = "admin";
    let password = "";
    let sudopassword = "";
    let file_key: FileList | null = null;
    let file_content = "";
  

    // Получаем значение query параметра 'type' из URL
    const type_data = derived(page, $page => $page.url.searchParams.get('type'));

    // Определяем, мастер это или слейв, на основе значения type

    function device_is_mester(): boolean {

        if ($type_data === 'master') {
            return true;
        } else {
            return false;
        }
    }
    

    console.log(type_data)
    

    // Функция для чтения содержимого файла и преобразования его в строку
    function readFileAsText(file: File): Promise<string> {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = () => resolve(reader.result as string);
            reader.onerror = reject;
            reader.readAsText(file);
        });
    }

    async function createDevice(event) {
        try {
            if (file_key && file_key.length > 0) {
                // Преобразуем первый файл в строку
                file_content = await readFileAsText(file_key[0]);
            }

            await invoke('create_device', {
                ipaddress: ip_address,
                port: port,
                username: username,
                password: password,
                sudopassword: sudopassword,
                fileKey: file_content,
                ismaster: device_is_mester()// Передаем содержимое файла как строку
            });

            // После успешного создания устройства перенаправляем на главную страницу
            window.location.href = '/';
        } catch (error) {
            console.error("Ошибка при создании устройства:", error);
        }
    }
</script>

<section class="vh-lg-100 mt-5 mt-lg-0 bg-soft d-flex align-items-center">
    <div class="container">
        <div class="row justify-content-center">
            <div class="col-12 d-flex align-items-center justify-content-center">
                <div class="bg-white shadow border-0 rounded border-light p-4 p-lg-5 w-100 fmxw-100">
                    <div class="text-center text-md-center mb-4 mt-md-0">
                        <h1 class="mb-0 h3">Добавить новый сервер</h1>
                    </div>
                    <form class="mt-4" on:submit|preventDefault={createDevice}>
                        <div class="form-group">
                            <div class="form-group mb-4">
                                <label for="host">Ip адрес</label>
                                <div class="input-group">
                                    <input type="text" placeholder="192.168.0.0" name="host" class="form-control" bind:value={ip_address}>
                                </div>
                            </div>
                            <div class="form-group mb-4">
                                <label for="port">Порт</label>
                                <div class="input-group">
                                    <input type="number" placeholder="Порт сервера" name="port" class="form-control" bind:value={port} step="1" max="65535" min="1">
                                </div>
                            </div>
                            <div class="form-group mb-4">
                                <label for="username">Пользователь</label>
                                <div class="input-group">
                                    <input type="text" placeholder="admin" name="username" class="form-control" bind:value={username}>
                                </div>
                            </div>
                            <div class="form-group mb-4">
                                <label for="password">Пароль</label>
                                <div class="input-group">
                                    <input type="text" placeholder="Если вы входите по ключу и пароль не требуется, не заполняйте это поле" name="password" class="form-control" bind:value={password}>
                                </div>
                            </div>
                            <div class="form-group mb-4">
                                <label for="sudopassword">Sudo Пароль</label>
                                <div class="input-group">
                                    <input type="text" placeholder="Если пароль пользователя совпадает с паролем sudo, можно не заполнять" name="sudopassword" class="form-control" bind:value={sudopassword}>
                                </div>
                            </div>
                            <div class="mb-3">
                                <label for="formFile" class="form-label">ssh ключ, файл</label>
                                <input type="file" class="form-control" name="file_key" bind:files={file_key}>
                            </div>
                        </div>
                        <div class="d-grid">
                            <button type="submit" class="btn btn-gray-800">Создать</button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>
</section>