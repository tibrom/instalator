<script lang="ts">
    import { invoke } from '@tauri-apps/api';
    import { onMount, } from 'svelte';
    
    // Определение типа AlertMessage, если его еще нет
    interface AlertMessage {
        belong: string; // или используйте правильный тип для `AlertType`
        message: string;
    }

    let messages: AlertMessage[] = []; // Инициализация массива с типом AlertMessage

    async function fetchMessages() {
        messages = await invoke<AlertMessage[]>('get_general_alert')
    }

    // Запрашиваем сообщения через регулярные интервалы
    setInterval(fetchMessages, 3000);
    //onMount(fetchMessages); // каждые 1000 мс (1 секунда)
</script>


<style>
    .scrollable-container {
        overflow-y: auto; /* Добавляет вертикальную полосу прокрутки при переполнении */
        height: 100% /* Устанавливает высоту на 100% от высоты родителя */
    }
</style>

<div class="container">
<h6>Уведомления</h6>

<div class="alert alert-dark scrollable-container" role="alert" style="height: 120px" >
    {#each messages as message}
        <div class="card" >
            <ul class="list-group list-group-flush">
                <li class="list-group-item text-primary">
                    {message.message}
                </li>
            </ul>
        </div>             <!-- Отобразите нужное свойство -->
    {/each}
</div>
</div>


