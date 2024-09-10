/**
* @typedef WkeEvent
* @property {string} name
* @property {any} data
* @property {number} timestamp
*/

/**
 * @typedef ResponseOk
 * @property {true} successed
 * @property {any} data
 */

/**
 * @typedef ResponseFail
 * @property {false} successed
 * @property {number} code
 * @property {string} message
 */

/**
 * 
 * @argument {string} version
 * @argument {any} nativeQuery
 */
return function (version, nativeQuery) {
    nativeQuery = nativeQuery.bind(nativeQuery);

    /**@type {Record<string, Array<(e: WkeEvent) => void>>} */
    const listeners = {};

    const ErrorCode = {
        Unknown: -1,
        NotImplements: -2,
        Timeout: -3,
        InvalidJson: -4
    };

    const timeoutFn = (timeout) => {
        return new Promise((_, reject) => {
            setTimeout(() => {
                reject({
                    code: ErrorCode.Timeout,
                    message: "timeout"
                });
            }, timeout);
        });
    };

    /**
     * 发送事件
     * @param {WkeEvent} e 
     */
    const emit = (e) => {
        const callbacks = listeners[e.name];
        if (callbacks) {
            callbacks.forEach(cb => {
                try {
                    cb(e);
                } catch (error) {
                    console.error(error);
                }
            });
        }
    };

    /**
     * 注册监听器
     * @param {string} name 
     * @param {(e: WkeEvent) => void} cb 
     */
    const on = (name, cb) => {
        let callbacks = listeners[name];
        if (!callbacks) {
            callbacks = listeners[name] = [];
        }
        callbacks.push(cb);
    };

    /**
     * 注销监听器
     * @param {string} name 
     * @param {(e: WkeEvent) => void} [cb] 
     */
    const off = (name, cb) => {
        if (typeof cb == "undefined") {
            delete listeners[name];
            return;
        }

        let callbacks = listeners[name];
        if (!callbacks)
            return;
        for (let i = 0; i < callbacks.length; ++i) {
            if (callbacks[i] == cb) {
                callbacks.splice(i, 1);
                break;
            }
        }
        if (callbacks.length == 0) {
            delete listeners[name];
            return;
        }
    };

    /**
     * 请求
     * @template REQ
     * @template RES
     * @param {string} name
     * @param {REQ} data
     * @returns {Promise<RES>}
     */
    const query = (name, data) => {
        /**@type {Promise<RES>} */
        const promise = new Promise((resolve, reject) => {
            resolve = function (resolve, content) {
                try {
                    resolve(JSON.parse(content));
                } catch (error) {
                    reject({ code: ErrorCode.InvalidJson, message: "invalid json" });
                }
            }.bind(null, resolve);

            reject = function (reject, reason) {
                if (typeof reason == "string") {
                    try {
                        reason = JSON.parse(reason);
                    } catch (error) {
                        console.error(error);
                    }
                }
                reject(reason);
            }.bind(null, reject);

            nativeQuery(
                JSON.stringify({ name, data }),
                resolve,
                reject
            );
        });

        /**
         * 超时失败
         * @param {Promise<RES>} promise 
         * @param {number} timeout 超时时间
         * @returns { Promise<RES> }
         */
        promise.timeout = function (promise, timeout) {
            return Promise.race(promise, timeoutFn(timeout));
        };
        return promise;
    };

    const wke = {};

    Object.defineProperty(wke, "name", {
        value: "wke",
        configurable: false,
        enumerable: true,
        writable: false,
    });

    Object.defineProperty(wke, "version", {
        value: version,
        configurable: false,
        enumerable: true,
        writable: false,
    });

    Object.defineProperty(wke, "ErrorCode", {
        value: ErrorCode,
        configurable: false,
        enumerable: true,
        writable: false,
    });

    Object.defineProperty(wke, "emit", {
        value: emit,
        configurable: false,
        enumerable: true,
        writable: false,
    });

    Object.defineProperty(wke, "on", {
        value: on,
        configurable: false,
        enumerable: true,
        writable: false,
    });

    Object.defineProperty(wke, "off", {
        value: off,
        configurable: false,
        enumerable: true,
        writable: false,
    });

    Object.defineProperty(wke, "query", {
        value: query,
        configurable: false,
        enumerable: true,
        writable: false,
    });

    Object.defineProperty(window, "wke", {
        value: wke,
        configurable: false,
        enumerable: true,
        writable: false,
    });
};