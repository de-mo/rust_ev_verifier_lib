export function useTimer() {
    let timer = null;

    const startTimer = (func, delay) => {
        timer = setInterval(func, delay)
    }

    const stopTimer = () => {
        if (timer) {
            clearInterval(timer);
            timer = null;
        }
    }

    return {startTimer, stopTimer}
}