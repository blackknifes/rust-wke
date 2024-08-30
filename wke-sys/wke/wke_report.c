#include "wke_report.h"
#include <stdio.h>

#define WKE_REPORT_ITERATOR(returnVal, name, ...) \
do { \
    char name_buf[128]; \
    sprintf(name_buf, "%s: %s\n", #name, name? "true" : "false"); \
    size_t name_len = strlen(name_buf); \
    if (wke_report_len + name_len > wke_report_capacity - 1) { \
        wke_report = (char*)realloc(wke_report, wke_report_capacity << 1); \
    } \
    strcat(wke_report, name_buf); \
    wke_report_len += name_len; \
} while(0);

static char* wke_report = (char*)0;
const char* wkeReport()
{
    if(wke_report)
        return wke_report;
    
    wke_report = (char*)malloc(0x4000);
    size_t wke_report_len = 0;
    size_t wke_report_capacity = 0;
    memset(wke_report, 0, 0x4000);

    WKE_FOR_EACH_DEFINE_FUNCTION(
        WKE_REPORT_ITERATOR, 
        WKE_REPORT_ITERATOR,
        WKE_REPORT_ITERATOR,
        WKE_REPORT_ITERATOR, 
        WKE_REPORT_ITERATOR, 
        WKE_REPORT_ITERATOR,
        WKE_REPORT_ITERATOR,
        WKE_REPORT_ITERATOR,
        WKE_REPORT_ITERATOR,
        WKE_REPORT_ITERATOR,
        WKE_REPORT_ITERATOR,
        WKE_REPORT_ITERATOR
    );
    return wke_report;
}
void wkeReportFree()
{
    if (!wke_report)
        return;
    free(wke_report);
    wke_report = (char*)0;
}