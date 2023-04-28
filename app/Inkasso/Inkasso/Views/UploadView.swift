//
//  UploadView.swift
//  Inkasso
//
//  Created by Ritam Sarmah on 3/23/23.
//

import SwiftUI

struct UploadView: View {
    
    @Environment(\.dismiss) var dismiss
    
    @ObservedObject var viewModel: UploadViewModel
    
    var body: some View {
        VStack {
            if let image = viewModel.image {
                HStack {
                    Spacer()
                    Image(uiImage: image)
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(maxHeight: 400)
                    Spacer()
                }
                .background(viewModel.useDarkBackground ? Color.black : Color.white)
            }
            
            Form {
                Section {
                    TextField("Title", text: $viewModel.title)
                    TextField("Artist", text: $viewModel.artist)
                    Toggle("Use Dark Background", isOn: $viewModel.useDarkBackground)
                }
                
                Section {
                    Toggle("Replace Existing", isOn: $viewModel.canOverwrite)
                }
                
                Button("Upload Artwork") {
                    Task {
                        if await viewModel.save() {
                            dismiss()
                        }
                    }
                }
                .disabled(viewModel.title.isEmpty)
            }
            .scrollDismissesKeyboard(.automatic)
        }
        .errorAlert(info: viewModel.errorInfo)
    }
}

//struct UploadView_Previews: PreviewProvider {
//    static var previews: some View {
//        UploadView()
//    }
//}
