#include "wke.h"
#include <Windows.h>


#if ENABLE_WKE != 1
void wkeSetWkeDllHandle(const HMODULE mainDllHandle)
{
    s_wkeMainDllHandle = mainDllHandle;
}

void wkeSetWkeDllPath(const wchar_t* dllPath)
{
    s_wkeDllPath = dllPath;
}

int wkeInitializeEx(const wkeSettings* settings)
{
    HMODULE hMod = s_wkeMainDllHandle;
    if (!hMod)
        hMod = LoadLibraryW(s_wkeDllPath);
    if (hMod) {
        FN_wkeInitializeEx wkeInitializeExFunc = (FN_wkeInitializeEx)GetProcAddress(hMod, "wkeInitializeEx");
        wkeInitializeExFunc(settings);

        WKE_FOR_EACH_DEFINE_FUNCTION(WKE_GET_PTR_ITERATOR0, WKE_GET_PTR_ITERATOR1, WKE_GET_PTR_ITERATOR2, WKE_GET_PTR_ITERATOR3, \
            WKE_GET_PTR_ITERATOR4, WKE_GET_PTR_ITERATOR5, WKE_GET_PTR_ITERATOR6, WKE_GET_PTR_ITERATOR9, WKE_GET_PTR_ITERATOR10, WKE_GET_PTR_ITERATOR11);
        return 1;
    }
    return 0;
}

int wkeInit()
{
    return wkeInitializeEx(((const wkeSettings*)0));
}

int wkeInitialize()
{
    return wkeInitializeEx(((const wkeSettings*)0));
}
#endif

void win32RunLoop()
{
    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)){
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
}

int win32RunLoopOnce()
{
    MSG msg;
    while (PeekMessage(&msg, NULL, 0, 0, PM_REMOVE)) {
        if (msg.message == WM_QUIT)
        {
            while (PeekMessage(&msg, NULL, 0, 0, PM_REMOVE))
            {
                TranslateMessage(&msg);
                DispatchMessage(&msg);        
            }
            return -1;
        }
        
        TranslateMessage(&msg);
        DispatchMessage(&msg);
        return 1;
    }

    return 0;
}

void win32ExitLoop()
{
    PostQuitMessage(0);
}