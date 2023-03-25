//
//  UploadView.swift
//  Picosso
//
//  Created by Ritam Sarmah on 3/23/23.
//

import SwiftUI

struct UploadView: View {
    
    @Environment(\.dismiss) var dismiss
    
    @ObservedObject var viewModel: UploadViewModel
    
    var body: some View {
        NavigationView {
            VStack {
                if let uiImage = viewModel.uiImage {
                    Image(uiImage: uiImage)
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(maxHeight: 400)
                }
                Form {
                    Section {
                        TextField("Title", text: $viewModel.title)
                        TextField("Artist", text: $viewModel.artist)
                        Toggle("Allow Padding", isOn: $viewModel.shouldPad)
                    }
                    Section {
                        Toggle("Replace Duplicate", isOn: $viewModel.shouldOverwrite)
                    }
                }
            }
            .navigationTitle("Add Artwork")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel", role: .cancel) {
                        dismiss()
                    }
                }
                
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Upload") {
                        Task {
                            let success = await viewModel.upload()
                            if success {
                                dismiss()
                            }
                        }
                    }
                    .disabled(viewModel.title.isEmpty)
                }
            }
        }
        .alert("Error", isPresented: $viewModel.isShowingErrorAlert, actions: {
            Button("OK", role: .cancel) {
                viewModel.errorMessage = nil
            }
        }, message: {
            Text(viewModel.errorMessage ?? "")
        })
    }
}

//struct UploadView_Previews: PreviewProvider {
//    static var previews: some View {
//        UploadView()
//    }
//}
