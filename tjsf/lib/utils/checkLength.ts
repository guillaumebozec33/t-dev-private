export function checkLength(str:any) {
    if(str && str.length >20){
        return str.slice(0,20) + "..."
    } else if (!str){
        return ""
    } else{
        return str
    }
}