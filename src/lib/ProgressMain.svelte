<script>
    // Ожидаем, что progress и maxStage будут переданы в виде пропсов
    export let progress = 0;
    export let maxStage = 100;

    

    $: percentage = (progress / maxStage) * 100;
</script>


<div class="progress">
    <div 
        class="progress-bar  bg-success progress-bar-striped" 
        role="progressbar" 
        style="width: {percentage}%" 
        aria-valuenow={percentage} 
        aria-valuemin="0" 
        aria-valuemax={maxStage}>
    </div>
</div>