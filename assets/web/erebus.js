/**
 * WebAssembly entry point. Resolves the canvas element by id, hands off to
 * `eframe::WebRunner` with the same `ErebusApp` the native binary uses.
 * @param {string} canvas_id
 */
export function start(canvas_id) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(canvas_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.start(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Window_b0c275b50676d397: function(arg0) {
            const ret = getObject(arg0).Window;
            return addHeapObject(ret);
        },
        __wbg_WorkerGlobalScope_7a1f78d9f7542cfa: function(arg0) {
            const ret = getObject(arg0).WorkerGlobalScope;
            return addHeapObject(ret);
        },
        __wbg___wbindgen_debug_string_ab4b34d23d6778bd: function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_in_a5d8b22e52b24dd1: function(arg0, arg1) {
            const ret = getObject(arg0) in getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_is_function_3baa9db1a987f47d: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_null_52ff4ec04186736f: function(arg0) {
            const ret = getObject(arg0) === null;
            return ret;
        },
        __wbg___wbindgen_is_object_63322ec0cd6ea4ef: function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_undefined_29a43b4d42920abd: function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        },
        __wbg___wbindgen_string_get_7ed5322991caaec5: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_6b64449b9b9ed33c: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_b46c9b5a9f08ec37: function(arg0) {
            getObject(arg0)._wbg_cb_unref();
        },
        __wbg_activeElement_737cd2e5ce862ac0: function(arg0) {
            const ret = getObject(arg0).activeElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_addEventListener_8176dab41b09531c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
        }, arguments); },
        __wbg_altKey_3116112ec764f316: function(arg0) {
            const ret = getObject(arg0).altKey;
            return ret;
        },
        __wbg_altKey_c4f26b40f1b826b4: function(arg0) {
            const ret = getObject(arg0).altKey;
            return ret;
        },
        __wbg_appendChild_e95c8b3b936d250a: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).appendChild(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_arrayBuffer_473644614d8643a5: function(arg0) {
            const ret = getObject(arg0).arrayBuffer();
            return addHeapObject(ret);
        },
        __wbg_at_03a250b9f2ce7aa5: function(arg0, arg1) {
            const ret = getObject(arg0).at(arg1);
            return addHeapObject(ret);
        },
        __wbg_beginComputePass_0fb772608bf84f44: function(arg0, arg1) {
            const ret = getObject(arg0).beginComputePass(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_beginRenderPass_c662486e5caabb09: function(arg0, arg1) {
            const ret = getObject(arg0).beginRenderPass(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_blockSize_9bfce6be11544dd1: function(arg0) {
            const ret = getObject(arg0).blockSize;
            return ret;
        },
        __wbg_blur_583010b6b4026c5d: function() { return handleError(function (arg0) {
            getObject(arg0).blur();
        }, arguments); },
        __wbg_body_c7b35a55457167ba: function(arg0) {
            const ret = getObject(arg0).body;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_bottom_12dded5de5311aff: function(arg0) {
            const ret = getObject(arg0).bottom;
            return ret;
        },
        __wbg_buffer_d0f5ea0926a691fd: function(arg0) {
            const ret = getObject(arg0).buffer;
            return addHeapObject(ret);
        },
        __wbg_button_c794bf4b1dcd7c4c: function(arg0) {
            const ret = getObject(arg0).button;
            return ret;
        },
        __wbg_cancelAnimationFrame_3fe3db137219c343: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).cancelAnimationFrame(arg1);
        }, arguments); },
        __wbg_changedTouches_6817cf10a2c671e4: function(arg0) {
            const ret = getObject(arg0).changedTouches;
            return addHeapObject(ret);
        },
        __wbg_clearBuffer_e063e34f4a181e05: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).clearBuffer(getObject(arg1), arg2, arg3);
        },
        __wbg_clearBuffer_f330030ddc7767fc: function(arg0, arg1, arg2) {
            getObject(arg0).clearBuffer(getObject(arg1), arg2);
        },
        __wbg_clearInterval_d04d8e0ff92c4c05: function(arg0, arg1) {
            getObject(arg0).clearInterval(arg1);
        },
        __wbg_click_bc40376705b1e04d: function(arg0) {
            getObject(arg0).click();
        },
        __wbg_clientX_48ead8c93aa7a872: function(arg0) {
            const ret = getObject(arg0).clientX;
            return ret;
        },
        __wbg_clientX_4b48f4fa9fb5d872: function(arg0) {
            const ret = getObject(arg0).clientX;
            return ret;
        },
        __wbg_clientY_014d3013b9b0c450: function(arg0) {
            const ret = getObject(arg0).clientY;
            return ret;
        },
        __wbg_clientY_ddcce7762c925e13: function(arg0) {
            const ret = getObject(arg0).clientY;
            return ret;
        },
        __wbg_clipboardData_f03e3b5606f47f6d: function(arg0) {
            const ret = getObject(arg0).clipboardData;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_clipboard_a08ffae077ba7949: function(arg0) {
            const ret = getObject(arg0).clipboard;
            return addHeapObject(ret);
        },
        __wbg_configure_c71c9f57ca3edf98: function(arg0, arg1) {
            getObject(arg0).configure(getObject(arg1));
        },
        __wbg_contentBoxSize_505997945c11aeab: function(arg0) {
            const ret = getObject(arg0).contentBoxSize;
            return addHeapObject(ret);
        },
        __wbg_contentRect_e3958925fadb3298: function(arg0) {
            const ret = getObject(arg0).contentRect;
            return addHeapObject(ret);
        },
        __wbg_copyBufferToBuffer_910ae8c201bdff01: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).copyBufferToBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
        },
        __wbg_copyBufferToTexture_8c287708aff282a4: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).copyBufferToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
        },
        __wbg_copyExternalImageToTexture_540fcadea7d8323f: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).copyExternalImageToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
        },
        __wbg_copyTextureToBuffer_76965133f36672a4: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).copyTextureToBuffer(getObject(arg1), getObject(arg2), getObject(arg3));
        },
        __wbg_copyTextureToTexture_04331d5254bea8fc: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).copyTextureToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
        },
        __wbg_createBindGroupLayout_fe258aa231f602a1: function(arg0, arg1) {
            const ret = getObject(arg0).createBindGroupLayout(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createBindGroup_783178b92eca4f94: function(arg0, arg1) {
            const ret = getObject(arg0).createBindGroup(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createBuffer_05c143bc69af7de1: function(arg0, arg1) {
            const ret = getObject(arg0).createBuffer(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createCommandEncoder_eeac00d01e7c7215: function(arg0, arg1) {
            const ret = getObject(arg0).createCommandEncoder(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createComputePipeline_70cb69a35311bb5a: function(arg0, arg1) {
            const ret = getObject(arg0).createComputePipeline(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createElement_bbd4c90086fe6f7b: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createObjectURL_46e1b0c55389893b: function() { return handleError(function (arg0, arg1) {
            const ret = URL.createObjectURL(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_createPipelineLayout_3195019c488e9d1f: function(arg0, arg1) {
            const ret = getObject(arg0).createPipelineLayout(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createQuerySet_a8afd88335f1ae22: function(arg0, arg1) {
            const ret = getObject(arg0).createQuerySet(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createRenderBundleEncoder_0ae4be9a26b4f4aa: function(arg0, arg1) {
            const ret = getObject(arg0).createRenderBundleEncoder(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createRenderPipeline_430c946fe289280f: function(arg0, arg1) {
            const ret = getObject(arg0).createRenderPipeline(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createSampler_59ee59f9ce9c89e6: function(arg0, arg1) {
            const ret = getObject(arg0).createSampler(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createShaderModule_cb92dd515bc68e5a: function(arg0, arg1) {
            const ret = getObject(arg0).createShaderModule(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createTexture_ae83ede28133180f: function(arg0, arg1) {
            const ret = getObject(arg0).createTexture(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_createView_c0fb516125a12571: function(arg0, arg1) {
            const ret = getObject(arg0).createView(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_ctrlKey_31968cccd46bdef6: function(arg0) {
            const ret = getObject(arg0).ctrlKey;
            return ret;
        },
        __wbg_ctrlKey_a49693667722b909: function(arg0) {
            const ret = getObject(arg0).ctrlKey;
            return ret;
        },
        __wbg_dataTransfer_5fa7598d8d6c3931: function(arg0) {
            const ret = getObject(arg0).dataTransfer;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_data_331de495b1c2eb08: function(arg0, arg1) {
            const ret = getObject(arg1).data;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_debug_c014a160490283dc: function(arg0) {
            console.debug(getObject(arg0));
        },
        __wbg_deltaMode_e3330902f10b9218: function(arg0) {
            const ret = getObject(arg0).deltaMode;
            return ret;
        },
        __wbg_deltaX_7f421a85054bc57c: function(arg0) {
            const ret = getObject(arg0).deltaX;
            return ret;
        },
        __wbg_deltaY_ca7744a8772482e1: function(arg0) {
            const ret = getObject(arg0).deltaY;
            return ret;
        },
        __wbg_destroy_d1537bee2b5a7849: function(arg0) {
            getObject(arg0).destroy();
        },
        __wbg_destroy_d28e196e9dbc3b27: function(arg0) {
            getObject(arg0).destroy();
        },
        __wbg_destroy_ddd5bee0b4b02f49: function(arg0) {
            getObject(arg0).destroy();
        },
        __wbg_devicePixelContentBoxSize_c1a8da18615df561: function(arg0) {
            const ret = getObject(arg0).devicePixelContentBoxSize;
            return addHeapObject(ret);
        },
        __wbg_devicePixelRatio_18e6533e6d7f4088: function(arg0) {
            const ret = getObject(arg0).devicePixelRatio;
            return ret;
        },
        __wbg_disconnect_d173374266b80cfa: function(arg0) {
            getObject(arg0).disconnect();
        },
        __wbg_dispatchWorkgroupsIndirect_e915df9199133ac5: function(arg0, arg1, arg2) {
            getObject(arg0).dispatchWorkgroupsIndirect(getObject(arg1), arg2);
        },
        __wbg_dispatchWorkgroups_0d71a3ed9fcaee9f: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).dispatchWorkgroups(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
        },
        __wbg_document_7a41071f2f439323: function(arg0) {
            const ret = getObject(arg0).document;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_drawIndexedIndirect_0954a720a9b13248: function(arg0, arg1, arg2) {
            getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
        },
        __wbg_drawIndexedIndirect_7882fca885de47ce: function(arg0, arg1, arg2) {
            getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
        },
        __wbg_drawIndexed_280977bb1d3baf3d: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
        },
        __wbg_drawIndexed_9a150a51a8427045: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
        },
        __wbg_drawIndirect_b393626eb70ae7fb: function(arg0, arg1, arg2) {
            getObject(arg0).drawIndirect(getObject(arg1), arg2);
        },
        __wbg_drawIndirect_c6c299eb2ddf8fd7: function(arg0, arg1, arg2) {
            getObject(arg0).drawIndirect(getObject(arg1), arg2);
        },
        __wbg_draw_26370233bc7d2e7e: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_draw_83285c3877561ec1: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_elementFromPoint_69f15053cea2ece4: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).elementFromPoint(arg1, arg2);
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_end_420d93a37f764933: function(arg0) {
            getObject(arg0).end();
        },
        __wbg_end_97a4259681c42d8d: function(arg0) {
            getObject(arg0).end();
        },
        __wbg_error_2001591ad2463697: function(arg0) {
            console.error(getObject(arg0));
        },
        __wbg_error_517d7241ee304d2d: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_error_d9a855c84f9b4e4c: function(arg0) {
            const ret = getObject(arg0).error;
            return addHeapObject(ret);
        },
        __wbg_executeBundles_452872ac4afbbf92: function(arg0, arg1) {
            getObject(arg0).executeBundles(getObject(arg1));
        },
        __wbg_features_15adc13e5b141301: function(arg0) {
            const ret = getObject(arg0).features;
            return addHeapObject(ret);
        },
        __wbg_features_f6c1f470639a88e2: function(arg0) {
            const ret = getObject(arg0).features;
            return addHeapObject(ret);
        },
        __wbg_files_68cba1b2e516e1ee: function(arg0) {
            const ret = getObject(arg0).files;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_finish_23cbd862d4229ec3: function(arg0, arg1) {
            const ret = getObject(arg0).finish(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_finish_52172eac54898d16: function(arg0) {
            const ret = getObject(arg0).finish();
            return addHeapObject(ret);
        },
        __wbg_finish_94bc184b535e2a90: function(arg0, arg1) {
            const ret = getObject(arg0).finish(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_finish_dad34d81d4500e85: function(arg0) {
            const ret = getObject(arg0).finish();
            return addHeapObject(ret);
        },
        __wbg_focus_089295847acbfa20: function() { return handleError(function (arg0) {
            getObject(arg0).focus();
        }, arguments); },
        __wbg_force_778963a43d577f61: function(arg0) {
            const ret = getObject(arg0).force;
            return ret;
        },
        __wbg_getBindGroupLayout_6d503a1fba524ee6: function(arg0, arg1) {
            const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_getBindGroupLayout_bc897888c0670dbe: function(arg0, arg1) {
            const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_getBoundingClientRect_ddac06b2c6b52b98: function(arg0) {
            const ret = getObject(arg0).getBoundingClientRect();
            return addHeapObject(ret);
        },
        __wbg_getCompilationInfo_469a33f449854be7: function(arg0) {
            const ret = getObject(arg0).getCompilationInfo();
            return addHeapObject(ret);
        },
        __wbg_getComputedStyle_a23c121719ab715c: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).getComputedStyle(getObject(arg1));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_getContext_69ddc504535a2e7b: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_getContext_fc146f8ec021d074: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_getCurrentTexture_e27b103ea7a3ce3c: function(arg0) {
            const ret = getObject(arg0).getCurrentTexture();
            return addHeapObject(ret);
        },
        __wbg_getData_a20c218e8ae28672: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getData(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getElementById_0b5a508c91194690: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_getItem_7fe1351b9ea3b2f3: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getMappedRange_4f36f39e059a63c6: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getMappedRange(arg1, arg2);
            return addHeapObject(ret);
        },
        __wbg_getPreferredCanvasFormat_13332df72e63723a: function(arg0) {
            const ret = getObject(arg0).getPreferredCanvasFormat();
            return (__wbindgen_enum_GpuTextureFormat.indexOf(ret) + 1 || 96) - 1;
        },
        __wbg_getPropertyValue_0bc8c6164d246228: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getPropertyValue(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_get_021cd0f12ed28063: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_get_0cfbe604d86bac03: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_get_3b0dcc1eb5df6032: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_get_8360291721e2339f: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        },
        __wbg_get_d4195ef4546b6d90: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_gpu_c773d7932dc745d7: function(arg0) {
            const ret = getObject(arg0).gpu;
            return addHeapObject(ret);
        },
        __wbg_has_b54bd7b6e9da11c7: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).has(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_hash_6b96fb5ff20f84b3: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).hash;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_height_528848d067cc2221: function(arg0) {
            const ret = getObject(arg0).height;
            return ret;
        },
        __wbg_height_f8efae863e677d02: function(arg0) {
            const ret = getObject(arg0).height;
            return ret;
        },
        __wbg_hidden_ace169f0d5c6f512: function(arg0) {
            const ret = getObject(arg0).hidden;
            return ret;
        },
        __wbg_host_b030f1d486d15963: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).host;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_hostname_87f5d71a94ee2d17: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).hostname;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_href_230fbde2e4776c03: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).href;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_id_8b383c92c097419c: function(arg0, arg1) {
            const ret = getObject(arg1).id;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_identifier_e18617399425f548: function(arg0) {
            const ret = getObject(arg0).identifier;
            return ret;
        },
        __wbg_info_7479429238bffbce: function(arg0) {
            console.info(getObject(arg0));
        },
        __wbg_inlineSize_ade7bedbe596e98c: function(arg0) {
            const ret = getObject(arg0).inlineSize;
            return ret;
        },
        __wbg_instanceof_Element_56c8d987654f359e: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Element;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuAdapter_0731153d2b08720b: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof GPUAdapter;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuCanvasContext_d14121c7bd72fcef: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof GPUCanvasContext;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuDeviceLostInfo_a3677ebb8241d800: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof GPUDeviceLostInfo;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuOutOfMemoryError_391d9a08edbfa04b: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof GPUOutOfMemoryError;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuValidationError_f4d803c383da3c92: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof GPUValidationError;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlAnchorElement_6231b5a29726bb69: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLAnchorElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlCanvasElement_ea4dfc3bb77c734b: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLCanvasElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlInputElement_8dc30e795ec4f2a5: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLInputElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Object_7c99480a1cdfb911: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Object;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_ResizeObserverEntry_cc1b4c1ded1b5810: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ResizeObserverEntry;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_ResizeObserverSize_9864626a3f8a20e4: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ResizeObserverSize;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_cc64c86c8ef9e02b: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_isComposing_15e96d5e3a32db4e: function(arg0) {
            const ret = getObject(arg0).isComposing;
            return ret;
        },
        __wbg_isComposing_d3c6bed96daefc5e: function(arg0) {
            const ret = getObject(arg0).isComposing;
            return ret;
        },
        __wbg_isSecureContext_3d130f603c477578: function(arg0) {
            const ret = getObject(arg0).isSecureContext;
            return ret;
        },
        __wbg_is_8f7ba86b7f249abd: function(arg0, arg1) {
            const ret = Object.is(getObject(arg0), getObject(arg1));
            return ret;
        },
        __wbg_item_acd72c282a80fdb4: function(arg0, arg1) {
            const ret = getObject(arg0).item(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_items_d12ee99dee134775: function(arg0) {
            const ret = getObject(arg0).items;
            return addHeapObject(ret);
        },
        __wbg_keyCode_972708a3ac86591a: function(arg0) {
            const ret = getObject(arg0).keyCode;
            return ret;
        },
        __wbg_key_2cbc38fa83cdb336: function(arg0, arg1) {
            const ret = getObject(arg1).key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_label_614ef5e608843844: function(arg0, arg1) {
            const ret = getObject(arg1).label;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_lastModified_0a38ca12ca1f1177: function(arg0) {
            const ret = getObject(arg0).lastModified;
            return ret;
        },
        __wbg_left_ea423c913972748d: function(arg0) {
            const ret = getObject(arg0).left;
            return ret;
        },
        __wbg_length_3d4ecd04bd8d22f1: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_length_5629b5731c1c71f4: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_length_72c35cb1847f1c54: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_length_9f1775224cf1d815: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_length_aa80af37af29c1dc: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_length_d34bf7d191aa0640: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_limits_2bfe39eb5f0b5a01: function(arg0) {
            const ret = getObject(arg0).limits;
            return addHeapObject(ret);
        },
        __wbg_limits_77193ad8b62f8502: function(arg0) {
            const ret = getObject(arg0).limits;
            return addHeapObject(ret);
        },
        __wbg_lineNum_95b780ade9fb4ba3: function(arg0) {
            const ret = getObject(arg0).lineNum;
            return ret;
        },
        __wbg_localStorage_f5f66b1ffd2486bc: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).localStorage;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_location_73c89ca5bb53ddf3: function(arg0) {
            const ret = getObject(arg0).location;
            return addHeapObject(ret);
        },
        __wbg_log_7e1aa9064a1dbdbd: function(arg0) {
            console.log(getObject(arg0));
        },
        __wbg_lost_21e9db8a9502a0ca: function(arg0) {
            const ret = getObject(arg0).lost;
            return addHeapObject(ret);
        },
        __wbg_mapAsync_f4fc38ac51855b15: function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).mapAsync(arg1 >>> 0, arg2, arg3);
            return addHeapObject(ret);
        },
        __wbg_matchMedia_ce9949babceac178: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).matchMedia(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_matches_60339f60d9118f30: function(arg0) {
            const ret = getObject(arg0).matches;
            return ret;
        },
        __wbg_matches_e0ed4cb607ce6402: function(arg0) {
            const ret = getObject(arg0).matches;
            return ret;
        },
        __wbg_maxBindGroups_cc0c1b6031ac310e: function(arg0) {
            const ret = getObject(arg0).maxBindGroups;
            return ret;
        },
        __wbg_maxBindingsPerBindGroup_d950de0c90e382e0: function(arg0) {
            const ret = getObject(arg0).maxBindingsPerBindGroup;
            return ret;
        },
        __wbg_maxBufferSize_01e5e024c304478a: function(arg0) {
            const ret = getObject(arg0).maxBufferSize;
            return ret;
        },
        __wbg_maxColorAttachmentBytesPerSample_91fc5eb9155186fd: function(arg0) {
            const ret = getObject(arg0).maxColorAttachmentBytesPerSample;
            return ret;
        },
        __wbg_maxColorAttachments_69f3bac8513cd2ce: function(arg0) {
            const ret = getObject(arg0).maxColorAttachments;
            return ret;
        },
        __wbg_maxComputeInvocationsPerWorkgroup_5d8e1f9e65b5443c: function(arg0) {
            const ret = getObject(arg0).maxComputeInvocationsPerWorkgroup;
            return ret;
        },
        __wbg_maxComputeWorkgroupSizeX_e8c75fa90e0b00b7: function(arg0) {
            const ret = getObject(arg0).maxComputeWorkgroupSizeX;
            return ret;
        },
        __wbg_maxComputeWorkgroupSizeY_72bce71ec7fa9330: function(arg0) {
            const ret = getObject(arg0).maxComputeWorkgroupSizeY;
            return ret;
        },
        __wbg_maxComputeWorkgroupSizeZ_8c7050ac47c80e42: function(arg0) {
            const ret = getObject(arg0).maxComputeWorkgroupSizeZ;
            return ret;
        },
        __wbg_maxComputeWorkgroupStorageSize_b789a39c5a0fd04a: function(arg0) {
            const ret = getObject(arg0).maxComputeWorkgroupStorageSize;
            return ret;
        },
        __wbg_maxComputeWorkgroupsPerDimension_a02a7f66f7c68b9c: function(arg0) {
            const ret = getObject(arg0).maxComputeWorkgroupsPerDimension;
            return ret;
        },
        __wbg_maxDynamicStorageBuffersPerPipelineLayout_90d4eb33665de8d1: function(arg0) {
            const ret = getObject(arg0).maxDynamicStorageBuffersPerPipelineLayout;
            return ret;
        },
        __wbg_maxDynamicUniformBuffersPerPipelineLayout_835864d8a793cc95: function(arg0) {
            const ret = getObject(arg0).maxDynamicUniformBuffersPerPipelineLayout;
            return ret;
        },
        __wbg_maxSampledTexturesPerShaderStage_f1fdaca8bd10047f: function(arg0) {
            const ret = getObject(arg0).maxSampledTexturesPerShaderStage;
            return ret;
        },
        __wbg_maxSamplersPerShaderStage_a0126ce660fc903a: function(arg0) {
            const ret = getObject(arg0).maxSamplersPerShaderStage;
            return ret;
        },
        __wbg_maxStorageBufferBindingSize_9ed12d54b564312c: function(arg0) {
            const ret = getObject(arg0).maxStorageBufferBindingSize;
            return ret;
        },
        __wbg_maxStorageBuffersPerShaderStage_7db5a7548c1199e6: function(arg0) {
            const ret = getObject(arg0).maxStorageBuffersPerShaderStage;
            return ret;
        },
        __wbg_maxStorageTexturesPerShaderStage_3df697d427690d26: function(arg0) {
            const ret = getObject(arg0).maxStorageTexturesPerShaderStage;
            return ret;
        },
        __wbg_maxTextureArrayLayers_759d0ac67e0a7d26: function(arg0) {
            const ret = getObject(arg0).maxTextureArrayLayers;
            return ret;
        },
        __wbg_maxTextureDimension1D_4bfdff8638ada7c1: function(arg0) {
            const ret = getObject(arg0).maxTextureDimension1D;
            return ret;
        },
        __wbg_maxTextureDimension2D_ea0c9c4d0b239666: function(arg0) {
            const ret = getObject(arg0).maxTextureDimension2D;
            return ret;
        },
        __wbg_maxTextureDimension3D_e76f3604806f47be: function(arg0) {
            const ret = getObject(arg0).maxTextureDimension3D;
            return ret;
        },
        __wbg_maxUniformBufferBindingSize_591ad000ffe10aad: function(arg0) {
            const ret = getObject(arg0).maxUniformBufferBindingSize;
            return ret;
        },
        __wbg_maxUniformBuffersPerShaderStage_6e5696dba506ca6c: function(arg0) {
            const ret = getObject(arg0).maxUniformBuffersPerShaderStage;
            return ret;
        },
        __wbg_maxVertexAttributes_fef434a4cf2ba188: function(arg0) {
            const ret = getObject(arg0).maxVertexAttributes;
            return ret;
        },
        __wbg_maxVertexBufferArrayStride_de60c38ec574b423: function(arg0) {
            const ret = getObject(arg0).maxVertexBufferArrayStride;
            return ret;
        },
        __wbg_maxVertexBuffers_d1a4a2fba06ae7d6: function(arg0) {
            const ret = getObject(arg0).maxVertexBuffers;
            return ret;
        },
        __wbg_message_8fd23df93c50075a: function(arg0, arg1) {
            const ret = getObject(arg1).message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_message_b00edacf4a520b03: function(arg0, arg1) {
            const ret = getObject(arg1).message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_message_d2eedafa0bd554a6: function(arg0, arg1) {
            const ret = getObject(arg1).message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_messages_1df11461d071c92c: function(arg0) {
            const ret = getObject(arg0).messages;
            return addHeapObject(ret);
        },
        __wbg_metaKey_665498d01ebfd062: function(arg0) {
            const ret = getObject(arg0).metaKey;
            return ret;
        },
        __wbg_metaKey_f8f3c1d2a5b88850: function(arg0) {
            const ret = getObject(arg0).metaKey;
            return ret;
        },
        __wbg_minStorageBufferOffsetAlignment_49f6b6baa1d34111: function(arg0) {
            const ret = getObject(arg0).minStorageBufferOffsetAlignment;
            return ret;
        },
        __wbg_minUniformBufferOffsetAlignment_39ec7837ddc9ee2c: function(arg0) {
            const ret = getObject(arg0).minUniformBufferOffsetAlignment;
            return ret;
        },
        __wbg_name_9edc86a6da7afd7a: function(arg0, arg1) {
            const ret = getObject(arg1).name;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_navigator_353318de944ca7f6: function(arg0) {
            const ret = getObject(arg0).navigator;
            return addHeapObject(ret);
        },
        __wbg_navigator_bc077756492232c5: function(arg0) {
            const ret = getObject(arg0).navigator;
            return addHeapObject(ret);
        },
        __wbg_new_0c7403db6e782f19: function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return addHeapObject(ret);
        },
        __wbg_new_682678e2f47e32bc: function() {
            const ret = new Array();
            return addHeapObject(ret);
        },
        __wbg_new_9fd25a23b3be27c1: function() {
            const ret = new Error();
            return addHeapObject(ret);
        },
        __wbg_new_aa8d0fa9762c29bd: function() {
            const ret = new Object();
            return addHeapObject(ret);
        },
        __wbg_new_ad8d9a2aa2624a65: function() { return handleError(function (arg0) {
            const ret = new ResizeObserver(getObject(arg0));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_new_from_slice_b5ea43e23f6008c0: function(arg0, arg1) {
            const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_new_with_byte_offset_and_length_01848e8d6a3d49ad: function(arg0, arg1, arg2) {
            const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_new_with_u8_array_sequence_and_options_afc143a3fe3b3456: function() { return handleError(function (arg0, arg1) {
            const ret = new Blob(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_now_36a3148ac47c4ad7: function(arg0) {
            const ret = getObject(arg0).now();
            return ret;
        },
        __wbg_now_e7c6795a7f81e10f: function(arg0) {
            const ret = getObject(arg0).now();
            return ret;
        },
        __wbg_observe_5ea88d68554155e1: function(arg0, arg1, arg2) {
            getObject(arg0).observe(getObject(arg1), getObject(arg2));
        },
        __wbg_offsetTop_551e185d17207caa: function(arg0) {
            const ret = getObject(arg0).offsetTop;
            return ret;
        },
        __wbg_offset_78dcfcd1f3ebc4ea: function(arg0) {
            const ret = getObject(arg0).offset;
            return ret;
        },
        __wbg_open_e7df9da99b95483f: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            const ret = getObject(arg0).open(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_origin_1f038926109a2a37: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).origin;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_performance_3fcf6e32a7e1ed0a: function(arg0) {
            const ret = getObject(arg0).performance;
            return addHeapObject(ret);
        },
        __wbg_performance_e0409977f06d6f6b: function(arg0) {
            const ret = getObject(arg0).performance;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_popErrorScope_efb23ea2dcc3b587: function(arg0) {
            const ret = getObject(arg0).popErrorScope();
            return addHeapObject(ret);
        },
        __wbg_port_3e514262af1a5056: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).port;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_preventDefault_f55c01cb5fd2bcc0: function(arg0) {
            getObject(arg0).preventDefault();
        },
        __wbg_protocol_bcb606858be99e32: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).protocol;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_prototypesetcall_a6b02eb00b0f4ce2: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
        },
        __wbg_pushErrorScope_9a7570b7a9f67657: function(arg0, arg1) {
            getObject(arg0).pushErrorScope(__wbindgen_enum_GpuErrorFilter[arg1]);
        },
        __wbg_push_471a5b068a5295f6: function(arg0, arg1) {
            const ret = getObject(arg0).push(getObject(arg1));
            return ret;
        },
        __wbg_querySelectorAll_e9e3fbd41310476e: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).querySelectorAll(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_queueMicrotask_5d15a957e6aa920e: function(arg0) {
            queueMicrotask(getObject(arg0));
        },
        __wbg_queueMicrotask_f8819e5ffc402f36: function(arg0) {
            const ret = getObject(arg0).queueMicrotask;
            return addHeapObject(ret);
        },
        __wbg_queue_9595c5175ef399b9: function(arg0) {
            const ret = getObject(arg0).queue;
            return addHeapObject(ret);
        },
        __wbg_reason_f9df4a653cfa764b: function(arg0) {
            const ret = getObject(arg0).reason;
            return (__wbindgen_enum_GpuDeviceLostReason.indexOf(ret) + 1 || 3) - 1;
        },
        __wbg_removeEventListener_7bdf07404d9b24bd: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
        }, arguments); },
        __wbg_remove_48cb93cf7a6c4260: function(arg0) {
            getObject(arg0).remove();
        },
        __wbg_requestAdapter_592f04f645dfaf68: function(arg0, arg1) {
            const ret = getObject(arg0).requestAdapter(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_requestAdapter_b11b709f5ffe7b21: function(arg0) {
            const ret = getObject(arg0).requestAdapter();
            return addHeapObject(ret);
        },
        __wbg_requestAnimationFrame_6f039d778639cc28: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
            return ret;
        }, arguments); },
        __wbg_requestDevice_52bb2980e6280ebc: function(arg0, arg1) {
            const ret = getObject(arg0).requestDevice(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_resolveQuerySet_b316102e1d152b52: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).resolveQuerySet(getObject(arg1), arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5 >>> 0);
        },
        __wbg_resolve_e6c466bc1052f16c: function(arg0) {
            const ret = Promise.resolve(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_right_6096346a1fca4d04: function(arg0) {
            const ret = getObject(arg0).right;
            return ret;
        },
        __wbg_search_ceee70e1153af3ec: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).search;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_setAttribute_6fde4098d274155c: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setBindGroup_0fb411b7d1ec4966: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
        },
        __wbg_setBindGroup_1c6bfc705c95f81f: function(arg0, arg1, arg2) {
            getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
        },
        __wbg_setBindGroup_2ec8db65419ec50c: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
        },
        __wbg_setBindGroup_3afbefd496741277: function(arg0, arg1, arg2) {
            getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
        },
        __wbg_setBindGroup_4ac51c0e16178380: function(arg0, arg1, arg2) {
            getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
        },
        __wbg_setBindGroup_c2fbfec522cc7572: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
        },
        __wbg_setBlendConstant_00bed453ac51c91b: function(arg0, arg1) {
            getObject(arg0).setBlendConstant(getObject(arg1));
        },
        __wbg_setIndexBuffer_42017bb879ab062b: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).setIndexBuffer(getObject(arg1), __wbindgen_enum_GpuIndexFormat[arg2], arg3);
        },
        __wbg_setIndexBuffer_4876c05f77106bb6: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setIndexBuffer(getObject(arg1), __wbindgen_enum_GpuIndexFormat[arg2], arg3, arg4);
        },
        __wbg_setIndexBuffer_8c79ee0b0b6460fa: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).setIndexBuffer(getObject(arg1), __wbindgen_enum_GpuIndexFormat[arg2], arg3);
        },
        __wbg_setIndexBuffer_e10a7cf5d063fdab: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setIndexBuffer(getObject(arg1), __wbindgen_enum_GpuIndexFormat[arg2], arg3, arg4);
        },
        __wbg_setItem_e6399d3faae141dc: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setPipeline_5c5a949bf12f8a5f: function(arg0, arg1) {
            getObject(arg0).setPipeline(getObject(arg1));
        },
        __wbg_setPipeline_c4793bebd98b8e56: function(arg0, arg1) {
            getObject(arg0).setPipeline(getObject(arg1));
        },
        __wbg_setPipeline_ce7a683c2c94919d: function(arg0, arg1) {
            getObject(arg0).setPipeline(getObject(arg1));
        },
        __wbg_setProperty_0d903d23a71dfe70: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setScissorRect_cf24179de05b8393: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setScissorRect(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_setStencilReference_7a98f054e2f31f54: function(arg0, arg1) {
            getObject(arg0).setStencilReference(arg1 >>> 0);
        },
        __wbg_setVertexBuffer_06dd033f8e75af24: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
        },
        __wbg_setVertexBuffer_c973cd35605098e4: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
        },
        __wbg_setVertexBuffer_e80315ecd1774568: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
        },
        __wbg_setVertexBuffer_ef41a6013dba1352: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
        },
        __wbg_setViewport_75637b1c9a301986: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            getObject(arg0).setViewport(arg1, arg2, arg3, arg4, arg5, arg6);
        },
        __wbg_set_022bee52d0b05b19: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments); },
        __wbg_set_autofocus_7aafe974868c0a0f: function(arg0, arg1) {
            getObject(arg0).autofocus = arg1 !== 0;
        },
        __wbg_set_box_e76b1c9ae3cbed18: function(arg0, arg1) {
            getObject(arg0).box = __wbindgen_enum_ResizeObserverBoxOptions[arg1];
        },
        __wbg_set_d8f1efe557b9e7e1: function(arg0, arg1, arg2) {
            getObject(arg0).set(getObject(arg1), arg2 >>> 0);
        },
        __wbg_set_download_f60111fb9c3eac2d: function(arg0, arg1, arg2) {
            getObject(arg0).download = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_height_24d07d982f176ac6: function(arg0, arg1) {
            getObject(arg0).height = arg1 >>> 0;
        },
        __wbg_set_height_be9b2b920bd68401: function(arg0, arg1) {
            getObject(arg0).height = arg1 >>> 0;
        },
        __wbg_set_href_f253ea395272f0c4: function(arg0, arg1, arg2) {
            getObject(arg0).href = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_onuncapturederror_5c20c4125b115c22: function(arg0, arg1) {
            getObject(arg0).onuncapturederror = getObject(arg1);
        },
        __wbg_set_tabIndex_f1f5240b07c77382: function(arg0, arg1) {
            getObject(arg0).tabIndex = arg1;
        },
        __wbg_set_type_15e4214f5c54262e: function(arg0, arg1, arg2) {
            getObject(arg0).type = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_type_8b2743f6b4de4035: function(arg0, arg1, arg2) {
            getObject(arg0).type = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_value_d84be184846d017b: function(arg0, arg1, arg2) {
            getObject(arg0).value = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_width_5cda41d4d06a14dd: function(arg0, arg1) {
            getObject(arg0).width = arg1 >>> 0;
        },
        __wbg_set_width_adc925bca9c5351a: function(arg0, arg1) {
            getObject(arg0).width = arg1 >>> 0;
        },
        __wbg_shiftKey_dcf8ee699c273ed2: function(arg0) {
            const ret = getObject(arg0).shiftKey;
            return ret;
        },
        __wbg_shiftKey_e483c13c966878f6: function(arg0) {
            const ret = getObject(arg0).shiftKey;
            return ret;
        },
        __wbg_size_2207e0a7b1f0c588: function(arg0) {
            const ret = getObject(arg0).size;
            return ret;
        },
        __wbg_size_b5c1b72884cb3fa5: function(arg0) {
            const ret = getObject(arg0).size;
            return ret;
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_stack_4f8b636eadc6cad8: function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_static_accessor_GLOBAL_8cfadc87a297ca02: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_602256ae5c8f42cf: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_SELF_e445c1c7484aecc3: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_WINDOW_f20e8576ef1e0f17: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_stopPropagation_e088fca8231e68c4: function(arg0) {
            getObject(arg0).stopPropagation();
        },
        __wbg_style_c331a9f6564f8f62: function(arg0) {
            const ret = getObject(arg0).style;
            return addHeapObject(ret);
        },
        __wbg_submit_6ffa2ed48b3eaecf: function(arg0, arg1) {
            getObject(arg0).submit(getObject(arg1));
        },
        __wbg_then_6701bb8428537e07: function(arg0, arg1) {
            const ret = getObject(arg0).then(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_then_792e0c862b060889: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        },
        __wbg_then_8e16ee11f05e4827: function(arg0, arg1) {
            const ret = getObject(arg0).then(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_then_a50dc2689b076063: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        },
        __wbg_top_158f7c4dd1427771: function(arg0) {
            const ret = getObject(arg0).top;
            return ret;
        },
        __wbg_touches_a66d38f0c03ba969: function(arg0) {
            const ret = getObject(arg0).touches;
            return addHeapObject(ret);
        },
        __wbg_type_94629e6712c72aa5: function(arg0, arg1) {
            const ret = getObject(arg1).type;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_type_ba6bfed8f5073b9e: function(arg0) {
            const ret = getObject(arg0).type;
            return (__wbindgen_enum_GpuCompilationMessageType.indexOf(ret) + 1 || 4) - 1;
        },
        __wbg_type_f9353a3f93f39cf6: function(arg0, arg1) {
            const ret = getObject(arg1).type;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_unmap_d610a495d70ebb5e: function(arg0) {
            getObject(arg0).unmap();
        },
        __wbg_usage_92ae9f7605bb82c1: function(arg0) {
            const ret = getObject(arg0).usage;
            return ret;
        },
        __wbg_userAgent_609f939440dc6b62: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).userAgent;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_valueOf_9760e35383abbb01: function(arg0) {
            const ret = getObject(arg0).valueOf();
            return addHeapObject(ret);
        },
        __wbg_value_6079dd28568d83c9: function(arg0, arg1) {
            const ret = getObject(arg1).value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_warn_3cc416af27dbdc02: function(arg0) {
            console.warn(getObject(arg0));
        },
        __wbg_width_5adcb07d04d08bdf: function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        },
        __wbg_width_ddbe321b233b5921: function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        },
        __wbg_writeBuffer_28f398e6955ad305: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).writeBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
        },
        __wbg_writeText_41e0b9b209591a06: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).writeText(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        },
        __wbg_writeTexture_4eafae0e29b3eac0: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).writeTexture(getObject(arg1), getObject(arg2), getObject(arg3), getObject(arg4));
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 437, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1099);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 932, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_4059);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Array<any>")], shim_idx: 298, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_821);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 298, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_821_3);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000005: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("GPUUncapturedErrorEvent")], shim_idx: 437, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1099_4);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000006: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 296, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_819);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000007: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000008: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
            const ret = getArrayU8FromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000009: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_object_clone_ref: function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./erebus_bg.js": import0,
    };
}

function __wasm_bindgen_func_elem_1099(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_1099(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_821(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_821(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_821_3(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_821_3(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_1099_4(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_1099_4(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_819(arg0, arg1) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.__wasm_bindgen_func_elem_819(retptr, arg0, arg1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

function __wasm_bindgen_func_elem_4059(arg0, arg1, arg2) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.__wasm_bindgen_func_elem_4059(retptr, arg0, arg1, addHeapObject(arg2));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}


const __wbindgen_enum_GpuCompilationMessageType = ["error", "warning", "info"];


const __wbindgen_enum_GpuDeviceLostReason = ["unknown", "destroyed"];


const __wbindgen_enum_GpuErrorFilter = ["validation", "out-of-memory", "internal"];


const __wbindgen_enum_GpuIndexFormat = ["uint16", "uint32"];


const __wbindgen_enum_GpuTextureFormat = ["r8unorm", "r8snorm", "r8uint", "r8sint", "r16uint", "r16sint", "r16float", "rg8unorm", "rg8snorm", "rg8uint", "rg8sint", "r32uint", "r32sint", "r32float", "rg16uint", "rg16sint", "rg16float", "rgba8unorm", "rgba8unorm-srgb", "rgba8snorm", "rgba8uint", "rgba8sint", "bgra8unorm", "bgra8unorm-srgb", "rgb9e5ufloat", "rgb10a2uint", "rgb10a2unorm", "rg11b10ufloat", "rg32uint", "rg32sint", "rg32float", "rgba16uint", "rgba16sint", "rgba16float", "rgba32uint", "rgba32sint", "rgba32float", "stencil8", "depth16unorm", "depth24plus", "depth24plus-stencil8", "depth32float", "depth32float-stencil8", "bc1-rgba-unorm", "bc1-rgba-unorm-srgb", "bc2-rgba-unorm", "bc2-rgba-unorm-srgb", "bc3-rgba-unorm", "bc3-rgba-unorm-srgb", "bc4-r-unorm", "bc4-r-snorm", "bc5-rg-unorm", "bc5-rg-snorm", "bc6h-rgb-ufloat", "bc6h-rgb-float", "bc7-rgba-unorm", "bc7-rgba-unorm-srgb", "etc2-rgb8unorm", "etc2-rgb8unorm-srgb", "etc2-rgb8a1unorm", "etc2-rgb8a1unorm-srgb", "etc2-rgba8unorm", "etc2-rgba8unorm-srgb", "eac-r11unorm", "eac-r11snorm", "eac-rg11unorm", "eac-rg11snorm", "astc-4x4-unorm", "astc-4x4-unorm-srgb", "astc-5x4-unorm", "astc-5x4-unorm-srgb", "astc-5x5-unorm", "astc-5x5-unorm-srgb", "astc-6x5-unorm", "astc-6x5-unorm-srgb", "astc-6x6-unorm", "astc-6x6-unorm-srgb", "astc-8x5-unorm", "astc-8x5-unorm-srgb", "astc-8x6-unorm", "astc-8x6-unorm-srgb", "astc-8x8-unorm", "astc-8x8-unorm-srgb", "astc-10x5-unorm", "astc-10x5-unorm-srgb", "astc-10x6-unorm", "astc-10x6-unorm-srgb", "astc-10x8-unorm", "astc-10x8-unorm-srgb", "astc-10x10-unorm", "astc-10x10-unorm-srgb", "astc-12x10-unorm", "astc-12x10-unorm-srgb", "astc-12x12-unorm", "astc-12x12-unorm-srgb"];


const __wbindgen_enum_ResizeObserverBoxOptions = ["border-box", "content-box", "device-pixel-content-box"];

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_export5(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export3(addHeapObject(e));
    }
}

let heap = new Array(1024).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_export5(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('erebus_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
