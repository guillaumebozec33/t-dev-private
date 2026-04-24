export function checkLength(str: string | null | undefined) {
    if (str && str.length > 20) {
        return str.slice(0, 20) + "...";
    }
    if (!str) {
        return "";
    }
    return str;
}