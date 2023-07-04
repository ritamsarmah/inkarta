//
//  ErrorAlert.swift
//  Inkarta
//
//  Created by Ritam Sarmah on 3/26/23.
//

import SwiftUI

struct ErrorAlert: ViewModifier {
    
    class Info: ObservableObject {
        @Published var isShowing = false
        
        var title: String = "Error"
        
        var message: String? {
            didSet {
                if oldValue == nil {
                    isShowing = true
                }
            }
        }
    }
    
    @State var info: Info
    
    func body(content: Content) -> some View {
        content
            .alert(info.title, isPresented: $info.isShowing, actions: {
                Button("OK", role: .cancel) {
                    info.message = nil
                }
            }, message: {
                Text(info.message ?? "")
            })
    }
}

extension View {
    func errorAlert(info: ErrorAlert.Info) -> some View {
        modifier(ErrorAlert(info: info))
    }
}
