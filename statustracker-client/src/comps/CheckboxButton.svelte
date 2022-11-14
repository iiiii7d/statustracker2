<script lang="ts">
  export let value = false;
  let button: HTMLButtonElement;

  function toggle() {
    value = !value;
    if (value) {
      button.setAttribute("checked", "true");
    } else {
      button.setAttribute("checked", "false");
    }
  }
</script>
<style lang="scss">
  button[checked=true] { 
    background-color: #8c0;
    color: #000;
    border: unset;
    box-shadow: #8c08 0px 3px;

    &:hover {
      background-color: #bf3;
      color: #000;
      box-shadow: #bf38 0px 4px;
    }
  }
</style>
<button on:click={toggle} bind:this={button} checked={value}><slot></slot></button>