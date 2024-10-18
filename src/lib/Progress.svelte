<script>
    // Ожидаем, что progress и maxStage будут переданы в виде пропсов
    export let progress = "Expectation 0";
    export let maxStage = 100;

    // Реактивные выражения для stage и percentage
    $: stage = progress && progress.startsWith("Expectation ") 
        ? parseInt(progress.replace("Expectation ", "")) 
        : 0;

    $: percentage = (stage / maxStage) * 100;

    // Для отладки
    console.log("progress:", progress);
    console.log("stage:", stage);
    console.log("percentage:", percentage);
</script>


<div class="progress">
    <div 
        class="progress-bar progress-bar-striped" 
        role="progressbar" 
        style="width: {percentage}%" 
        aria-valuenow={percentage} 
        aria-valuemin="0" 
        aria-valuemax={maxStage}>
    </div>
</div>

