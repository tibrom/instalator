<script>
    // Ожидаем, что progress и maxStage будут переданы в виде пропсов
    export let progressMaster = "Expectation 0";
    export let progressSlave = "Expectation 0";
    export let maxStage = 100;

    // Реактивные выражения для stage и percentage
    $: stageMaster = progressMaster && progressMaster.startsWith("Expectation ") 
        ? parseInt(progressMaster.replace("Expectation ", "")) 
        : 0;
    $: stageSlave = progressSlave && progressSlave.startsWith("Expectation ") 
        ? parseInt(progressSlave.replace("Expectation ", "")) 
        : 0;

    $: percentage = ((stageMaster + stageSlave)/ maxStage) * 100;
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
